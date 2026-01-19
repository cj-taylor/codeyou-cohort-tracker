const API_BASE = "";
let currentClassId = null;
let allClasses = [];
let demoMode = true; // Demo mode on by default
let previousSectionModal = null; // Track which section modal was open
let assignmentNameMap = {}; // Map assignment IDs to their display names for demo mode

// Escape function for inline event handlers
function escapeForAttr(str) {
  return String(str)
    .replace(/\\/g, '\\\\')
    .replace(/'/g, "\\'");
}

// Demo mode functions
function toggleDemoMode() {
  const classListCheckbox = document.getElementById("demo-mode");
  const dashboardCheckbox = document.getElementById("demo-mode-dashboard");
  
  // Get state from whichever checkbox exists and was clicked
  if (classListCheckbox) {
    demoMode = classListCheckbox.checked;
  }
  if (dashboardCheckbox) {
    demoMode = dashboardCheckbox.checked;
  }
  
  // Sync the other checkbox if it exists
  if (classListCheckbox && dashboardCheckbox) {
    classListCheckbox.checked = demoMode;
    dashboardCheckbox.checked = demoMode;
  }
  
  // Reload current view if on dashboard
  if (currentClassId) {
    init();
  }
  // No need to reload class list - it doesn't show student data
}

function maskName(firstName, lastName, index) {
  if (!demoMode) return `${firstName} ${lastName}`;
  return `Student ${index + 1}`;
}

function maskEmail(email, index) {
  if (!demoMode) return email;
  return `student${index + 1}@example.com`;
}

function maskRegion(region) {
  if (!demoMode) return region;
  return "Region";
}

function maskClassName(className, index) {
  if (!demoMode) return className;
  return `Class ${index + 1}`;
}

function maskAssignmentName(assignmentName, index) {
  if (!demoMode) return assignmentName;
  // Determine type from original name if possible
  const lowerName = assignmentName.toLowerCase();
  if (lowerName.includes('quiz')) return `Quiz ${index + 1}`;
  if (lowerName.includes('project')) return `Project ${index + 1}`;
  if (lowerName.includes('exercise')) return `Exercise ${index + 1}`;
  return `Assignment ${index + 1}`;
}

function maskSectionName(sectionName, index) {
  if (!demoMode) return sectionName;
  return `Section ${index + 1}`;
}

// Class Selection Functions
async function loadClasses() {
  try {
    const showArchived = document.getElementById("show-archived").checked;
    const endpoint = showArchived ? "/classes?all=true" : "/classes";
    const classes = await fetchData(endpoint);

    if (!classes || classes.length === 0) {
      document.getElementById("class-grid").innerHTML =
        '<div style="grid-column: 1/-1; text-align: center; padding: 40px; color: #666;">No classes found. Run <code>cargo run -- init</code> to set up classes.</div>';
      return;
    }

    allClasses = classes;
    renderClassGrid(classes);
  } catch (error) {
    console.error("Error loading classes:", error);
    document.getElementById("class-grid").innerHTML =
      '<div class="error" style="grid-column: 1/-1;">Failed to load classes. Make sure the server is running.</div>';
  }
}

function renderClassGrid(classes) {
  const grid = document.getElementById("class-grid");
  grid.innerHTML = classes
    .map((cls, index) => {
      const syncedDate = cls.synced_at
        ? new Date(cls.synced_at).toLocaleString()
        : "Never synced";
      const displayName = maskClassName(cls.name, index);
      const displayFriendlyId = demoMode ? `class-${index + 1}` : cls.friendly_id;
      return `
                    <div class="class-card ${cls.is_active ? "" : "inactive"}">
                        <div onclick="selectClass('${cls.id}', '${
        cls.name
      }')" style="cursor: pointer;">
                            <h3>${displayName}</h3>
                            <div class="friendly-id">${displayFriendlyId}</div>
                            <div class="status">${
                              cls.is_active ? "✓ Active" : "○ Inactive"
                            }</div>
                            <div class="synced-at">Last synced: ${syncedDate}</div>
                        </div>
                        <div style="margin-top: 10px; display: flex; gap: 5px;">
                            <button onclick="event.stopPropagation(); syncClass('${
                              cls.friendly_id
                            }')" 
                                    style="flex: 1; padding: 6px; background: #667eea; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 0.85rem;">
                                Sync
                            </button>
                            <button onclick="event.stopPropagation(); toggleClassActive('${
                              cls.id
                            }', ${cls.is_active})" 
                                    style="flex: 1; padding: 6px; background: ${
                                      cls.is_active ? "#ef4444" : "#10b981"
                                    }; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 0.85rem;">
                                ${cls.is_active ? "Deactivate" : "Activate"}
                            </button>
                        </div>
                        <div id="sync-status-${
                          cls.friendly_id
                        }" style="margin-top: 5px; font-size: 0.8rem; color: #666;"></div>
                    </div>
                `;
    })
    .join("");
}

async function toggleClassActive(classId, isCurrentlyActive) {
  const action = isCurrentlyActive ? "deactivate" : "activate";
  try {
    const response = await fetch(`${API_BASE}/classes/${classId}/${action}`, {
      method: "POST",
    });
    if (!response.ok) throw new Error(`Failed to ${action}`);
    await loadClasses();
  } catch (error) {
    alert(`Failed to ${action} class: ${error.message}`);
  }
}

async function syncClass(classId) {
  const statusEl = document.getElementById(`sync-status-${classId}`);
  statusEl.textContent = "Starting sync...";
  statusEl.style.color = "#667eea";

  const eventSource = new EventSource(`${API_BASE}/classes/${classId}/sync`);

  eventSource.onmessage = (event) => {
    statusEl.textContent = event.data;
    statusEl.style.color = "#667eea";
  };

  eventSource.onerror = () => {
    eventSource.close();
    if (statusEl.textContent.includes("complete")) {
      statusEl.style.color = "#10b981";
      setTimeout(async () => {
        statusEl.textContent = "";
        await loadClasses(); // Reload to get updated synced_at
      }, 2000);
    } else {
      statusEl.textContent = "✗ Sync failed";
      statusEl.style.color = "#ef4444";
    }
  };
}

function toggleArchivedClasses() {
  loadClasses();
}

function selectClass(classId, className) {
  currentClassId = classId;
  localStorage.setItem("selectedClassId", classId);
  localStorage.setItem("selectedClassName", className);

  document.getElementById("class-selection").style.display = "none";
  document.getElementById("dashboard").style.display = "block";
  
  // Find the class index for masking
  const classIndex = allClasses.findIndex(c => c.id === classId);
  const displayName = maskClassName(className, classIndex);
  document.getElementById("class-name").textContent = displayName;

  init();
}

function backToClasses() {
  document.getElementById("dashboard").style.display = "none";
  document.getElementById("class-selection").style.display = "block";
  currentClassId = null;
  localStorage.removeItem("selectedClassId");
  localStorage.removeItem("selectedClassName");
  loadClasses();
}

// Check for previously selected class
function checkStoredClass() {
  const storedClassId = localStorage.getItem("selectedClassId");
  const storedClassName = localStorage.getItem("selectedClassName");

  if (storedClassId && storedClassName) {
    selectClass(storedClassId, storedClassName);
  } else {
    loadClasses();
  }
}

async function fetchData(endpoint) {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return await response.json();
  } catch (error) {
    console.error(`Error fetching ${endpoint}:`, error);
    return null;
  }
}

function formatPercent(value) {
  return (value * 100).toFixed(1) + "%";
}

function formatDate(timestamp) {
  if (!timestamp) return "Never";
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

async function loadHealth() {
  const data = await fetchData("/health");
  if (!data) return;

  document.getElementById("sync-info").textContent = `Last sync: ${formatDate(
    data.last_sync
  )}`;
}

async function loadSummary() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/progress-summary?night=${globalNightFilter}`
    : `/classes/${currentClassId}/progress-summary`;
  const data = await fetchData(endpoint);
  if (!data) return;

  // Update class-specific counts
  document.getElementById("stat-students").textContent = data.total_students;
  document.getElementById("stat-assignments").textContent =
    data.total_assignments;
  document.getElementById("stat-progressions").textContent =
    data.total_progressions.toLocaleString();

  document.getElementById("stat-completion").textContent = formatPercent(
    data.completion_rate
  );
  document.getElementById("stat-avg-grade").textContent = data.avg_grade
    ? formatPercent(data.avg_grade)
    : "N/A";
}

async function loadAssignmentDifficulty() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/assignment-difficulty?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/assignment-difficulty`;
  const data = await fetchData(endpoint);
  const tbody = document.getElementById("difficulty-body");

  if (!data || data.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6">No data available</td></tr>';
    return;
  }

  tbody.innerHTML = data
    .slice(0, 10)
    .map((item, index) => {
      // Color code difficulty: red (high), yellow (medium), green (low)
      let difficultyColor = "#10b981"; // green
      if (item.difficulty_score > 0.6) difficultyColor = "#ef4444"; // red
      else if (item.difficulty_score > 0.4) difficultyColor = "#fbbf24"; // yellow
      
      const displayName = maskAssignmentName(item.name, index);
      const displaySection = item.section ? maskSectionName(item.section, index) : "-";
      assignmentNameMap[item.assignment_id] = displayName; // Store for modal use
      
      return `
                <tr style="cursor:pointer;" onclick="openAssignmentModal('${
                  item.assignment_id
                }', '${escapeForAttr(displayName)}')">
                    <td>${displaySection}</td>
                    <td title="${displayName}">${
        displayName.length > 30 ? displayName.substring(0, 30) + "..." : displayName
      }</td>
                    <td style="text-transform: capitalize;">${item.assignment_type}</td>
                    <td>
                        <div style="display:flex; align-items:center; gap:10px;">
                            <div class="progress-bar" style="width:60px;">
                                <div class="fill" style="width:${
                                  item.difficulty_score * 100
                                }%; background: ${difficultyColor};"></div>
                            </div>
                            <span style="color: ${difficultyColor}; font-weight: 600;">${(item.difficulty_score * 100).toFixed(0)}</span>
                        </div>
                    </td>
                    <td>
                        <div style="display:flex; align-items:center; gap:10px;">
                            <div class="progress-bar" style="width:60px;">
                                <div class="fill" style="width:${
                                  item.completion_rate * 100
                                }%"></div>
                            </div>
                            <span>${formatPercent(item.completion_rate)}</span>
                        </div>
                    </td>
                    <td>${
                      item.avg_grade ? formatPercent(item.avg_grade) : "N/A"
                    }</td>
                </tr>
                `;
    })
    .join("");
}

// Keep old loadBlockers for backward compatibility but point to new function
async function loadBlockers() {
  return loadAssignmentDifficulty();
}

async function loadStudentHealth() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/student-health?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/student-health`;
  const data = await fetchData(endpoint);
  const tbody = document.getElementById("risk-body");

  if (!data || data.length === 0) {
    tbody.innerHTML = '<tr><td colspan="3">No data available</td></tr>';
    return;
  }

  // Show only at-risk students (not low risk)
  const atRisk = data.filter((s) => s.risk !== "low").slice(0, 10);

  if (atRisk.length === 0) {
    tbody.innerHTML =
      '<tr><td colspan="3" style="color:#16a34a;">All students on track!</td></tr>';
    return;
  }

  tbody.innerHTML = atRisk
    .map(
      (student, index) => `
                <tr style="cursor:pointer;" onclick="openStudentModal('${
                  student.student_id
                }')">
                    <td>${maskName(student.first_name, student.last_name, index)}</td>
                    <td>
                        <div style="display:flex; align-items:center; gap:10px;">
                            <div class="progress-bar" style="width:80px;">
                                <div class="fill" style="width:${
                                  student.completion_pct * 100
                                }%"></div>
                            </div>
                            <span>${student.completed}/${
        student.total_assignments
      }</span>
                        </div>
                    </td>
                    <td><span class="risk-badge risk-${student.risk}">${
        student.risk
      }</span></td>
                </tr>
            `
    )
    .join("");
}

// Store student data for search filtering
let studentActivityData = [];

async function loadStudentActivity() {
  // Use global night filter
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/student-activity?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/student-activity`;

  const data = await fetchData(endpoint);
  const tbody = document.getElementById("activity-body");

  if (!data || data.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6">No data available</td></tr>';
    studentActivityData = [];
    return;
  }

  studentActivityData = data;
  renderStudentActivityTable(data);
}

function filterStudentTable() {
  const searchTerm = document
    .getElementById("student-search")
    .value.toLowerCase();
  if (!searchTerm) {
    renderStudentActivityTable(studentActivityData);
    return;
  }
  const filtered = studentActivityData.filter(
    (s) =>
      `${s.first_name} ${s.last_name}`.toLowerCase().includes(searchTerm) ||
      s.email.toLowerCase().includes(searchTerm)
  );
  renderStudentActivityTable(filtered);
}

function renderStudentActivityTable(data) {
  const tbody = document.getElementById("activity-body");

  if (!data || data.length === 0) {
    tbody.innerHTML = '<tr><td colspan="6">No students found</td></tr>';
    return;
  }

  function formatActivityDate(dateStr) {
    if (!dateStr) return "Never";
    const date = new Date(dateStr);
    return date.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function getActivityClass(days) {
    if (days === null || days === undefined) return "activity-inactive";
    if (days <= 7) return "activity-recent";
    if (days <= 14) return "activity-warning";
    return "activity-inactive";
  }

  function getActivityLabel(days) {
    if (days === null || days === undefined) return "No activity";
    if (days === 0) return "Today";
    if (days === 1) return "1 day";
    return `${days} days`;
  }

  tbody.innerHTML = data
    .map(
      (student, index) => `
                <tr style="cursor:pointer;" onclick="openStudentModal('${student.student_id}')">
                    <td>
                        <div>${maskName(student.first_name, student.last_name, index)}</div>
                        <div style="font-size:0.8rem; color:#666;">${maskEmail(
                          student.email, index
                        )}</div>
                    </td>
                    <td>${student.night || "-"}</td>
                    <td>
                        <div style="display:flex; align-items:center; gap:10px;">
                            <div class="progress-bar" style="width:80px;">
                                <div class="fill" style="width:${
                                  (student.total_completions /
                                    student.total_assignments) *
                                  100
                                }%"></div>
                            </div>
                            <span>${student.total_completions}/${
        student.total_assignments
      } (${Math.round(
        (student.total_completions / student.total_assignments) * 100
      )}%)</span>
                        </div>
                    </td>
                    <td>${formatActivityDate(student.last_activity)}</td>
                    <td>
                        <span class="activity-badge ${getActivityClass(
                          student.days_inactive
                        )}">
                            ${getActivityLabel(student.days_inactive)}
                        </span>
                    </td>
                </tr>
            `
    )
    .join("");
}

// Student Modal Functions
let studentProgressChart = null;
let studentDayChart = null;
let studentTimeChart = null;

async function openStudentModal(studentId) {
  const modal = document.getElementById("student-modal");
  modal.style.display = "flex";

  // Load student detail
  const detail = await fetchData(
    `/classes/${currentClassId}/students/${studentId}/detail`
  );
  if (!detail) {
    alert("Failed to load student details");
    closeStudentModal();
    return;
  }

  // Find student index for masking
  const studentIndex = studentActivityData.findIndex(s => s.student_id === studentId);

  // Update header
  document.getElementById(
    "modal-student-name"
  ).textContent = maskName(detail.first_name, detail.last_name, studentIndex);

  // Update info
  document.getElementById("modal-student-info").innerHTML = `${
    maskEmail(detail.email, studentIndex)
  } | ${detail.night || "No night assigned"} Night | ${
    maskRegion(detail.region || "No region")
  }`;

  // Update stats
  const riskClass =
    detail.risk === "low"
      ? "success"
      : detail.risk === "medium"
      ? "warning"
      : "danger";
  document.getElementById("modal-stats").innerHTML = `
                <div class="modal-stat">
                    <div class="value">${formatPercent(
                      detail.completion_pct
                    )}</div>
                    <div class="label">Completion</div>
                </div>
                <div class="modal-stat ${
                  detail.avg_grade && detail.avg_grade >= 0.7
                    ? "success"
                    : "warning"
                }">
                    <div class="value">${
                      detail.avg_grade ? formatPercent(detail.avg_grade) : "N/A"
                    }</div>
                    <div class="label">Avg Grade</div>
                </div>
                <div class="modal-stat ${
                  detail.days_inactive <= 7
                    ? "success"
                    : detail.days_inactive <= 14
                    ? "warning"
                    : "danger"
                }">
                    <div class="value">${
                      detail.days_inactive !== null
                        ? detail.days_inactive + "d"
                        : "-"
                    }</div>
                    <div class="label">Days Inactive</div>
                </div>
                <div class="modal-stat ${riskClass}">
                    <div class="value">${detail.risk.toUpperCase()}</div>
                    <div class="label">Risk Level</div>
                </div>
            `;

  // Load assignments and timeline in parallel
  const [assignments, timeline] = await Promise.all([
    fetchData(`/classes/${currentClassId}/students/${studentId}/assignments`),
    fetchData(
      `/classes/${currentClassId}/students/${studentId}/progress-timeline`
    ),
  ]);

  // Render problem areas
  if (assignments) {
    const incomplete = assignments.filter((a) => !a.completed);
    const lowGrade = assignments.filter(
      (a) => a.completed && a.grade && a.grade < 0.7
    );

    const problemAreas = document.getElementById("problem-areas");
    if (incomplete.length === 0 && lowGrade.length === 0) {
      problemAreas.innerHTML =
        '<p style="color:#16a34a;">No problem areas identified!</p>';
    } else {
      let html = "";
      incomplete.slice(0, 5).forEach((a, idx) => {
        const displayName = maskAssignmentName(a.name, idx);
        html += `<div class="problem-item incomplete">Not completed: ${displayName}</div>`;
      });
      lowGrade.slice(0, 5).forEach((a, idx) => {
        const displayName = maskAssignmentName(a.name, incomplete.length + idx);
        html += `<div class="problem-item low-grade">Low grade (${formatPercent(
          a.grade
        )}): ${displayName}</div>`;
      });
      if (incomplete.length > 5) {
        html += `<div style="color:#666; font-size:0.85rem;">...and ${
          incomplete.length - 5
        } more incomplete</div>`;
      }
      problemAreas.innerHTML = html;
    }

    // Render assignment list
    const assignmentList = document.getElementById("assignment-list");

    assignmentList.innerHTML = assignments
      .map((a, idx) => {
        let statusClass, statusText;
        if (!a.completed) {
          statusClass = "status-missing";
          statusText = "Missing";
        } else if (a.grade && a.grade < 0.7) {
          statusClass = "status-low";
          statusText = "Low";
        } else {
          statusClass = "status-done";
          statusText = "Done";
        }

        const completedDate = a.completed_at
          ? new Date(a.completed_at).toLocaleDateString("en-US", {
              month: "short",
              day: "numeric",
              year: "numeric",
            })
          : "-";

        const displayName = maskAssignmentName(a.name, idx);
        const displaySection = a.section ? maskSectionName(a.section, idx) : "-";

        return `
                        <tr>
                            <td>${displaySection}</td>
                            <td>${displayName}</td>
                            <td><span class="assignment-type">${
                              a.assignment_type
                            }</span></td>
                            <td class="${statusClass}">${statusText}</td>
                            <td>${a.grade ? formatPercent(a.grade) : "-"}</td>
                            <td>${completedDate}</td>
                        </tr>
                    `;
      })
      .join("");

    // Initialize sorting for this table
    initModalTableSorting();
  }

  // Render progress chart
  if (timeline && timeline.length > 0) {
    const ctx = document
      .getElementById("student-progress-chart")
      .getContext("2d");

    // Destroy existing chart if any
    if (studentProgressChart) {
      studentProgressChart.destroy();
    }

    // Helper to convert ISO week to date range
    function getWeekDateRange(isoWeek) {
      const [year, week] = isoWeek.split("-");
      const weekNum = parseInt(week);

      const jan1 = new Date(parseInt(year), 0, 1);
      const daysToMonday = (8 - jan1.getDay()) % 7;
      const firstMonday = new Date(jan1);
      firstMonday.setDate(jan1.getDate() + daysToMonday);

      const weekStart = new Date(firstMonday);
      weekStart.setDate(firstMonday.getDate() + (weekNum - 1) * 7);
      const weekEnd = new Date(weekStart);
      weekEnd.setDate(weekStart.getDate() + 6);

      const fmt = (d) =>
        d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
      return `${fmt(weekStart)} - ${fmt(weekEnd)}`;
    }

    studentProgressChart = new Chart(ctx, {
      type: "bar",
      data: {
        labels: timeline.map((d) => getWeekDateRange(d.week)),
        datasets: [
          {
            label: "Weekly Completions",
            type: "bar",
            data: timeline.map((d) => d.completed),
            backgroundColor: "rgba(102, 126, 234, 0.6)",
            borderColor: "#667eea",
            borderWidth: 1,
          },
          {
            label: "Cumulative Total",
            type: "line",
            data: timeline.map((d) => d.cumulative),
            borderColor: "#10b981",
            backgroundColor: "transparent",
            borderWidth: 2,
            tension: 0.3,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: true,
            position: "top",
          },
        },
        scales: {
          y: { beginAtZero: true },
        },
      },
    });
  }

  // Load and render day of week chart
  const dayData = await fetchData(
    `/classes/${currentClassId}/students/${studentId}/day-of-week`
  );
  if (dayData && dayData.length > 0) {
    const ctx = document.getElementById("student-day-chart").getContext("2d");

    if (studentDayChart) {
      studentDayChart.destroy();
    }

    const days = [
      "Sunday",
      "Monday",
      "Tuesday",
      "Wednesday",
      "Thursday",
      "Friday",
      "Saturday",
    ];
    const dayMap = {};
    dayData.forEach((d) => (dayMap[d.day] = d.count));
    const counts = days.map((day) => dayMap[day] || 0);

    studentDayChart = new Chart(ctx, {
      type: "bar",
      data: {
        labels: days,
        datasets: [
          {
            label: "Completions",
            data: counts,
            backgroundColor: "rgba(102, 126, 234, 0.6)",
            borderColor: "#667eea",
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { display: false },
        },
        scales: {
          y: {
            beginAtZero: true,
            ticks: { precision: 0 },
          },
        },
      },
    });
  }

  // Load and render time of day chart
  const timeData = await fetchData(
    `/classes/${currentClassId}/students/${studentId}/time-of-day`
  );
  if (timeData && timeData.length > 0) {
    const ctx = document.getElementById("student-time-chart").getContext("2d");

    if (studentTimeChart) {
      studentTimeChart.destroy();
    }

    const periods = [
      "Morning (6am-12pm)",
      "Afternoon (12pm-6pm)",
      "Evening (6pm-12am)",
      "Night (12am-6am)",
    ];
    const periodMap = {};
    timeData.forEach((d) => (periodMap[d.day] = d.count));
    const counts = periods.map((period) => periodMap[period] || 0);

    studentTimeChart = new Chart(ctx, {
      type: "bar",
      data: {
        labels: periods,
        datasets: [
          {
            label: "Completions",
            data: counts,
            backgroundColor: "rgba(102, 126, 234, 0.6)",
            borderColor: "#667eea",
            borderWidth: 1,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { display: false },
        },
        scales: {
          y: {
            beginAtZero: true,
            ticks: { precision: 0 },
          },
        },
      },
    });
  }
}

function closeStudentModal() {
  document.getElementById("student-modal").style.display = "none";
  if (studentProgressChart) {
    studentProgressChart.destroy();
    studentProgressChart = null;
  }
  if (studentDayChart) {
    studentDayChart.destroy();
    studentDayChart = null;
  }
  if (studentTimeChart) {
    studentTimeChart.destroy();
    studentTimeChart = null;
  }
  
  // Reopen section modal if it was open before
  if (previousSectionModal) {
    const sectionName = previousSectionModal;
    previousSectionModal = null;
    openSectionModal(sectionName);
  }
}

// Close modal on outside click
document
  .getElementById("student-modal")
  .addEventListener("click", function (e) {
    if (e.target === this) {
      closeStudentModal();
    }
  });

// Assignment Modal Functions
async function openAssignmentModal(assignmentId, assignmentName) {
  const modal = document.getElementById("assignment-modal");
  modal.style.display = "flex";

  document.getElementById("modal-assignment-name").textContent = assignmentName;
  document.getElementById("modal-assignment-info").textContent =
    "Loading student completion data...";

  // Fetch all students and their status for this assignment
  let students = await fetchData(`/classes/${currentClassId}/students`);
  const progressions = await fetchData(
    `/classes/${currentClassId}/progressions`
  );

  if (!students || !progressions) {
    document.getElementById("modal-assignment-info").textContent =
      "Failed to load data";
    return;
  }

  // Apply night filter if active
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  
  if (globalNightFilter && globalNightFilter !== "") {
    students = students.filter(s => s.night && s.night.toLowerCase() === globalNightFilter.toLowerCase());
  }

  // Build completion map (only for filtered students)
  const studentIds = new Set(students.map(s => s.id));
  const completionMap = {};
  progressions.forEach((p) => {
    if (p.assignment_id === assignmentId && studentIds.has(p.student_id)) {
      completionMap[p.student_id] = {
        completed: true,
        grade: p.grade,
        completed_at: p.completed_at,
      };
    }
  });

  // Count completions
  const completed = Object.keys(completionMap).length;
  const total = students.length;
  document.getElementById(
    "modal-assignment-info"
  ).innerHTML = `${completed} of ${total} students completed (${formatPercent(
    completed / total
  )})`;

  // Render student list
  const tbody = document.getElementById("assignment-students-body");
  tbody.innerHTML = students
    .map((student, index) => {
      const completion = completionMap[student.id];
      const statusClass = completion ? "status-done" : "status-missing";
      const statusText = completion ? "Completed" : "Not Started";
      const grade = completion?.grade ? formatPercent(completion.grade) : "-";
      const completedDate = completion?.completed_at
        ? new Date(completion.completed_at).toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
            year: "numeric",
          })
        : "-";

      return `
                    <tr>
                        <td>${maskName(student.first_name, student.last_name, index)}</td>
                        <td class="${statusClass}">${statusText}</td>
                        <td>${grade}</td>
                        <td>${completedDate}</td>
                    </tr>
                `;
    })
    .join("");

  // Initialize sorting for this table
  initAssignmentModalTableSorting();
}

function closeAssignmentModal() {
  document.getElementById("assignment-modal").style.display = "none";
}

// Close assignment modal on outside click
document
  .getElementById("assignment-modal")
  .addEventListener("click", function (e) {
    if (e.target === this) {
      closeAssignmentModal();
    }
  });

async function loadProgressChart() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/progress-over-time?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/progress-over-time`;
  const data = await fetchData(endpoint);
  if (!data || data.length === 0) return;

  const ctx = document.getElementById("progress-chart").getContext("2d");

  // Helper to convert ISO week (YYYY-WW) to date range
  function getWeekDateRange(isoWeek) {
    const [year, week] = isoWeek.split("-");
    const weekNum = parseInt(week);

    // Get first day of year
    const jan1 = new Date(parseInt(year), 0, 1);
    // Find first Monday of the year
    const daysToMonday = (8 - jan1.getDay()) % 7;
    const firstMonday = new Date(jan1);
    firstMonday.setDate(jan1.getDate() + daysToMonday);

    // Calculate week start
    const weekStart = new Date(firstMonday);
    weekStart.setDate(firstMonday.getDate() + (weekNum - 1) * 7);
    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekStart.getDate() + 6);

    const fmt = (d) =>
      d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
    return `${fmt(weekStart)} - ${fmt(weekEnd)}`;
  }

  new Chart(ctx, {
    type: "line",
    data: {
      labels: data.map((d) => getWeekDateRange(d.week)),
      datasets: [
        {
          label: "Weekly Completions",
          type: "bar",
          data: data.map((d) => d.completed),
          backgroundColor: "rgba(102, 126, 234, 0.6)",
          borderColor: "#667eea",
          borderWidth: 1,
        },
        {
          label: "Cumulative Total",
          type: "line",
          data: data.map((d) => d.cumulative),
          borderColor: "#10b981",
          backgroundColor: "transparent",
          borderWidth: 2,
          tension: 0.3,
          yAxisID: "y1",
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: {
        intersect: false,
        mode: "index",
      },
      plugins: {
        legend: {
          position: "top",
        },
      },
      scales: {
        y: {
          beginAtZero: true,
          title: {
            display: true,
            text: "Weekly",
          },
        },
        y1: {
          position: "right",
          beginAtZero: true,
          title: {
            display: true,
            text: "Cumulative",
          },
          grid: {
            drawOnChartArea: false,
          },
        },
      },
      onClick: (event, elements) => {
        if (elements.length > 0) {
          const index = elements[0].index;
          const weekData = data[index];
          openWeekModal(weekData.week);
        }
      },
    },
  });
}

async function openWeekModal(isoWeek) {
  const modal = document.getElementById("week-modal");
  modal.style.display = "flex";

  // Convert ISO week to date range
  const [year, week] = isoWeek.split('-');
  const weekNum = parseInt(week);
  const jan1 = new Date(parseInt(year), 0, 1);
  const daysToMonday = (8 - jan1.getDay()) % 7;
  const firstMonday = new Date(jan1);
  firstMonday.setDate(jan1.getDate() + daysToMonday);
  const weekStart = new Date(firstMonday);
  weekStart.setDate(firstMonday.getDate() + (weekNum - 1) * 7);
  const weekEnd = new Date(weekStart);
  weekEnd.setDate(weekStart.getDate() + 6);
  
  const fmt = (d) => d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
  const dateRange = `${fmt(weekStart)} - ${fmt(weekEnd)}`;

  document.getElementById("modal-week-name").textContent = `Week ${weekNum} (${dateRange})`;
  document.getElementById("modal-week-info").textContent = "Loading...";

  // Fetch progressions for this week
  const progressions = await fetchData(`/classes/${currentClassId}/progressions`);
  const assignments = await fetchData(`/classes/${currentClassId}/assignments`);
  let students = await fetchData(`/classes/${currentClassId}/students`);

  if (!progressions || !assignments || !students) {
    document.getElementById("modal-week-info").textContent = "Failed to load data";
    return;
  }

  // Apply night filter if active
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  let filteredStudentIds = null;
  if (globalNightFilter) {
    students = students.filter(s => s.night && s.night.toLowerCase() === globalNightFilter.toLowerCase());
    filteredStudentIds = new Set(students.map(s => s.id));
  }

  // Filter progressions for this week and by night if applicable
  const weekProgressions = progressions.filter(p => {
    if (!p.completed_at) return false;
    if (filteredStudentIds && !filteredStudentIds.has(p.student_id)) return false;
    const completedWeek = new Date(p.completed_at).toISOString().split('T')[0];
    const completedDate = new Date(completedWeek);
    return completedDate >= weekStart && completedDate <= weekEnd;
  });

  // Count completions by assignment
  const assignmentCounts = {};
  weekProgressions.forEach(p => {
    assignmentCounts[p.assignment_id] = (assignmentCounts[p.assignment_id] || 0) + 1;
  });

  // Build assignment map
  const assignmentMap = {};
  assignments.forEach(a => {
    assignmentMap[a.id] = a;
  });

  document.getElementById("modal-week-info").textContent = 
    `${weekProgressions.length} total completions across ${Object.keys(assignmentCounts).length} assignments`;

  // Render assignment list
  const tbody = document.getElementById("week-assignments-body");
  const sortedAssignments = Object.entries(assignmentCounts)
    .sort((a, b) => b[1] - a[1])
    .map(([assignmentId, count]) => ({
      assignment: assignmentMap[assignmentId],
      count
    }));

  tbody.innerHTML = sortedAssignments
    .map(({ assignment, count }, idx) => {
      if (!assignment) return '';
      const displayName = maskAssignmentName(assignment.name, idx);
      const displaySection = assignment.section ? maskSectionName(assignment.section, idx) : '-';
      return `
        <tr>
          <td>${displaySection}</td>
          <td>${displayName}</td>
          <td>${count}</td>
        </tr>
      `;
    })
    .join("");
    
  // Initialize sorting for this table
  initWeekModalTableSorting();
}

function initWeekModalTableSorting() {
  const table = document.getElementById("week-assignments-table");
  const headers = table.querySelectorAll("th");
  
  headers.forEach((header, index) => {
    header.classList.add("sortable");
    header.onclick = () => {
      const tbody = table.querySelector("tbody");
      const rows = Array.from(tbody.querySelectorAll("tr"));
      
      // Determine sort direction
      const isAsc = header.classList.contains("sort-asc");
      headers.forEach(h => h.classList.remove("sort-asc", "sort-desc"));
      header.classList.add(isAsc ? "sort-desc" : "sort-asc");
      
      // Sort rows
      rows.sort((a, b) => {
        const aVal = a.cells[index].textContent.trim();
        const bVal = b.cells[index].textContent.trim();
        
        // Try numeric comparison for count column
        if (index === 2) {
          return isAsc ? bVal - aVal : aVal - bVal;
        }
        
        // String comparison
        return isAsc 
          ? bVal.localeCompare(aVal)
          : aVal.localeCompare(bVal);
      });
      
      // Re-append sorted rows
      rows.forEach(row => tbody.appendChild(row));
    };
  });
}

function closeWeekModal() {
  document.getElementById("week-modal").style.display = "none";
}

document.getElementById("week-modal").addEventListener("click", function(e) {
  if (e.target === this) {
    closeWeekModal();
  }
});

async function loadDayOfWeekChart() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/day-of-week?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/day-of-week`;
  const data = await fetchData(endpoint);
  if (!data || data.length === 0) return;

  const ctx = document.getElementById("day-of-week-chart").getContext("2d");
  const days = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
  ];

  // Fill in missing days with 0
  const dayMap = {};
  data.forEach((d) => (dayMap[d.day] = d.count));
  const counts = days.map((day) => dayMap[day] || 0);

  new Chart(ctx, {
    type: "bar",
    data: {
      labels: days,
      datasets: [
        {
          label: "Completions",
          data: counts,
          backgroundColor: "rgba(102, 126, 234, 0.6)",
          borderColor: "#667eea",
          borderWidth: 1,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: false },
      },
      scales: {
        y: {
          beginAtZero: true,
          ticks: { precision: 0 },
        },
      },
    },
  });
}

async function loadTimeOfDayChart() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/time-of-day?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/time-of-day`;
  const data = await fetchData(endpoint);
  if (!data || data.length === 0) return;

  const ctx = document.getElementById("time-of-day-chart").getContext("2d");
  const periods = [
    "Morning (6am-12pm)",
    "Afternoon (12pm-6pm)",
    "Evening (6pm-12am)",
    "Night (12am-6am)",
  ];

  const periodMap = {};
  data.forEach((d) => (periodMap[d.day] = d.count));
  const counts = periods.map((period) => periodMap[period] || 0);

  new Chart(ctx, {
    type: "bar",
    data: {
      labels: periods,
      datasets: [
        {
          label: "Completions",
          data: counts,
          backgroundColor: "rgba(102, 126, 234, 0.6)",
          borderColor: "#667eea",
          borderWidth: 1,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: false },
      },
      scales: {
        y: {
          beginAtZero: true,
          ticks: { precision: 0 },
        },
      },
    },
  });
}

async function loadSectionProgress() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/section-progress?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/section-progress`;
  const data = await fetchData(endpoint);
  const tbody = document.getElementById("section-progress-body");

  if (!data || data.length === 0) {
    tbody.innerHTML = '<tr><td colspan="4">No section data available</td></tr>';
    return;
  }

  tbody.innerHTML = data
    .map((section, index) => {
      const startedRate = (section.students_started / section.total_students) * 100;
      const completedRate = (section.students_completed / section.total_students) * 100;
      const completionRate = section.students_started > 0
        ? (section.students_completed / section.students_started) * 100
        : 0;
      
      const displaySection = maskSectionName(section.section, index);
      
      return `
        <tr style="cursor:pointer;" onclick="openSectionModal('${escapeForAttr(section.section)}')">
          <td>${displaySection}</td>
          <td data-value="${section.students_started}">
            <div style="display:flex; align-items:center; gap:10px;">
              <div class="progress-bar" style="width:80px;">
                <div class="fill" style="width:${startedRate}%"></div>
              </div>
              <span>${section.students_started}/${section.total_students} (${Math.round(startedRate)}%)</span>
            </div>
          </td>
          <td data-value="${section.students_completed}">
            <div style="display:flex; align-items:center; gap:10px;">
              <div class="progress-bar" style="width:80px;">
                <div class="fill" style="width:${completedRate}%"></div>
              </div>
              <span>${section.students_completed}/${section.total_students} (${Math.round(completedRate)}%)</span>
            </div>
          </td>
          <td data-value="${completionRate}">
            <div style="display:flex; align-items:center; gap:10px;">
              <div class="progress-bar" style="width:80px;">
                <div class="fill" style="width:${completionRate}%"></div>
              </div>
              <span>${Math.round(completionRate)}%</span>
            </div>
          </td>
        </tr>
      `;
    })
    .join("");
    
  // Initialize sorting for this table
  initSectionProgressTableSorting();
}

function initSectionProgressTableSorting() {
  const table = document.getElementById("section-progress-table");
  const headers = table.querySelectorAll("th");
  
  headers.forEach((header, index) => {
    header.classList.add("sortable");
    header.onclick = () => {
      const tbody = table.querySelector("tbody");
      const rows = Array.from(tbody.querySelectorAll("tr"));
      
      // Determine sort direction
      const isAsc = header.classList.contains("sort-asc");
      headers.forEach(h => h.classList.remove("sort-asc", "sort-desc"));
      header.classList.add(isAsc ? "sort-desc" : "sort-asc");
      
      // Sort rows
      rows.sort((a, b) => {
        let aVal, bVal;
        
        if (index === 0) {
          // Section name - string comparison
          aVal = a.cells[index].textContent.trim();
          bVal = b.cells[index].textContent.trim();
          return isAsc 
            ? bVal.localeCompare(aVal)
            : aVal.localeCompare(bVal);
        } else {
          // Numeric columns - use data-value attribute
          aVal = parseFloat(a.cells[index].getAttribute('data-value')) || 0;
          bVal = parseFloat(b.cells[index].getAttribute('data-value')) || 0;
          return isAsc ? bVal - aVal : aVal - bVal;
        }
      });
      
      // Re-append sorted rows
      rows.forEach(row => tbody.appendChild(row));
    };
  });
}

async function loadNightSummary() {
  const data = await fetchData(
    `/classes/${currentClassId}/metrics/night-summary`
  );
  const container = document.getElementById("night-summary-container");

  if (!data || data.length === 0) {
    container.innerHTML =
      '<p style="color:#666;">No night data available. Import student nights using the CLI.</p>';
    return;
  }

  container.innerHTML = `<div class="night-cards">
                ${data
                  .map(
                    (night) => `
                    <div class="night-card">
                        <h3>${night.night} Night</h3>
                        <div class="stat-row">
                            <span class="stat-label">Students</span>
                            <span class="stat-value">${
                              night.student_count
                            }</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">Total Completions</span>
                            <span class="stat-value">${night.total_completions.toLocaleString()}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">Avg Completion</span>
                            <span class="stat-value">${formatPercent(
                              night.avg_completion_pct
                            )}</span>
                        </div>
                        <div class="stat-row">
                            <span class="stat-label">Avg Grade</span>
                            <span class="stat-value">${
                              night.avg_grade
                                ? formatPercent(night.avg_grade)
                                : "N/A"
                            }</span>
                        </div>
                        ${
                          night.mentors && night.mentors.length > 0
                            ? `
                            <div class="mentors">
                                <strong>Mentors:</strong> ${
                                  demoMode 
                                    ? night.mentors.map((_, i) => `Mentor ${i + 1}`).join(", ")
                                    : night.mentors.join(", ")
                                }
                            </div>
                        `
                            : ""
                        }
                    </div>
                `
                  )
                  .join("")}
            </div>`;
}

async function loadAssignmentTypes() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/assignment-types?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/assignment-types`;
  const data = await fetchData(endpoint);
  const tbody = document.getElementById("assignment-type-body");

  if (!data || data.length === 0) {
    tbody.innerHTML = '<tr><td colspan="4">No data available</td></tr>';
    return;
  }

  tbody.innerHTML = data
    .map((type) => {
      return `
        <tr>
          <td style="text-transform: capitalize;">${type.assignment_type}</td>
          <td>${type.total_assignments}</td>
          <td>
            <div style="display:flex; align-items:center; gap:10px;">
              <div class="progress-bar" style="width:80px;">
                <div class="fill" style="width:${type.avg_completion_rate * 100}%"></div>
              </div>
              <span>${formatPercent(type.avg_completion_rate)}</span>
            </div>
          </td>
          <td>${type.avg_grade ? formatPercent(type.avg_grade) : "N/A"}</td>
        </tr>
      `;
    })
    .join("");
}

async function loadGradeDistribution() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/grade-distribution?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/grade-distribution`;
  const data = await fetchData(endpoint);
  
  if (!data || data.length === 0) return;

  const ctx = document.getElementById("grade-distribution-chart").getContext("2d");
  
  // Destroy existing chart if any
  const existingChart = Chart.getChart(ctx);
  if (existingChart) {
    existingChart.destroy();
  }

  new Chart(ctx, {
    type: "bar",
    data: {
      labels: data.map(d => d.range),
      datasets: [
        {
          label: "Number of Grades",
          data: data.map(d => d.count),
          backgroundColor: "rgba(102, 126, 234, 0.6)",
          borderColor: "#667eea",
          borderWidth: 1,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: false },
        tooltip: {
          callbacks: {
            label: function(context) {
              const item = data[context.dataIndex];
              return `${item.count} grades (${item.percentage.toFixed(1)}%)`;
            }
          }
        }
      },
      scales: {
        y: {
          beginAtZero: true,
          ticks: { precision: 0 },
          title: {
            display: true,
            text: "Count"
          }
        },
        x: {
          title: {
            display: true,
            text: "Grade Range"
          }
        }
      },
    },
  });
}

async function loadVelocityChart() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/velocity?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/velocity`;
  const data = await fetchData(endpoint);
  
  if (!data || data.length === 0) return;

  const ctx = document.getElementById("velocity-chart").getContext("2d");
  
  // Destroy existing chart if any
  const existingChart = Chart.getChart(ctx);
  if (existingChart) {
    existingChart.destroy();
  }

  // Helper to convert ISO week to date range
  function getWeekDateRange(isoWeek) {
    const [year, week] = isoWeek.split("-");
    const weekNum = parseInt(week);
    const jan1 = new Date(parseInt(year), 0, 1);
    const daysToMonday = (8 - jan1.getDay()) % 7;
    const firstMonday = new Date(jan1);
    firstMonday.setDate(jan1.getDate() + daysToMonday);
    const weekStart = new Date(firstMonday);
    weekStart.setDate(firstMonday.getDate() + (weekNum - 1) * 7);
    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekStart.getDate() + 6);
    const fmt = (d) =>
      d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
    return `${fmt(weekStart)} - ${fmt(weekEnd)}`;
  }

  new Chart(ctx, {
    type: "line",
    data: {
      labels: data.map(d => getWeekDateRange(d.week)),
      datasets: [
        {
          label: "Avg Assignments/Student/Week",
          data: data.map(d => d.avg_completions_per_student),
          borderColor: "#10b981",
          backgroundColor: "rgba(16, 185, 129, 0.1)",
          borderWidth: 2,
          tension: 0.3,
          fill: true,
        },
      ],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      plugins: {
        legend: { display: true },
        tooltip: {
          callbacks: {
            label: function(context) {
              const item = data[context.dataIndex];
              return [
                `Avg: ${item.avg_completions_per_student.toFixed(1)} assignments/student`,
                `Total: ${item.total_completions} completions`,
                `Active: ${item.active_students} students`
              ];
            }
          }
        }
      },
      scales: {
        y: {
          beginAtZero: true,
          title: {
            display: true,
            text: "Assignments per Student"
          }
        }
      },
    },
  });
}

async function loadEngagementGaps() {
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  const endpoint = globalNightFilter
    ? `/classes/${currentClassId}/metrics/engagement-gaps?night=${globalNightFilter}`
    : `/classes/${currentClassId}/metrics/engagement-gaps`;
  const data = await fetchData(endpoint);
  
  const alert = document.getElementById("engagement-gap-alert");
  const list = document.getElementById("engagement-gap-list");
  
  if (!data || data.length === 0) {
    alert.style.display = "none";
    return;
  }
  
  alert.style.display = "block";
  list.innerHTML = data.slice(0, 5).map((student, index) => `
    <div style="padding: 8px; background: white; margin-bottom: 8px; border-radius: 4px; display: flex; justify-content: space-between; align-items: center; cursor: pointer;" onclick="openStudentModal('${student.student_id}')">
      <div>
        <strong>${maskName(student.first_name, student.last_name, index)}</strong>
        <span style="color: #666; font-size: 0.85rem; margin-left: 10px;">${student.night || 'No night'}</span>
      </div>
      <div style="text-align: right;">
        <span style="color: #856404; font-weight: 600;">${student.days_inactive} days inactive</span>
        <span style="color: #666; font-size: 0.85rem; margin-left: 10px;">${Math.round(student.completion_pct * 100)}% complete</span>
      </div>
    </div>
  `).join('');
  
  if (data.length > 5) {
    list.innerHTML += `<p style="color: #856404; font-size: 0.85rem; margin-top: 8px;">...and ${data.length - 5} more students</p>`;
  }
}

// Load all data
async function init() {
  if (!currentClassId) return;

  await Promise.all([
    loadHealth(),
    loadSummary(),
    loadBlockers(),
    loadStudentHealth(),
    loadStudentActivity(),
    loadEngagementGaps(),
    loadNightSummary(),
    loadAssignmentTypes(),
    loadGradeDistribution(),
    loadVelocityChart(),
    loadProgressChart(),
    loadDayOfWeekChart(),
    loadTimeOfDayChart(),
    loadSectionProgress(),
    setupNightFilter(),
  ]);

  // Initialize table sorting after data loads
  initTableSorting();
}

async function setupNightFilter() {
  // Get all students to check for night data
  const students = await fetchData(`/classes/${currentClassId}/students`);
  
  if (!students || students.length === 0) return;
  
  // Check if any students have night data
  const hasNightData = students.some(s => s.night && s.night.trim() !== '');
  
  if (!hasNightData) {
    document.getElementById('night-filter-message').style.display = 'block';
    return;
  }
  
  // Get unique nights
  const nights = [...new Set(students.map(s => s.night).filter(n => n && n.trim() !== ''))].sort();
  
  // Populate filter - clear existing options first except "All Nights"
  const select = document.getElementById('global-night-filter');
  while (select.options.length > 1) {
    select.remove(1);
  }
  
  nights.forEach(night => {
    const option = document.createElement('option');
    option.value = night;
    option.textContent = night;
    select.appendChild(option);
  });
  
  // Restore saved night filter
  const savedNight = localStorage.getItem(`nightFilter_${currentClassId}`);
  if (savedNight && nights.includes(savedNight)) {
    select.value = savedNight;
  }
  
  document.getElementById('night-filter-container').style.display = 'block';
}

function applyNightFilter() {
  const selectedNight = document.getElementById('global-night-filter').value;
  
  // Save selection to localStorage
  if (selectedNight) {
    localStorage.setItem(`nightFilter_${currentClassId}`, selectedNight);
  } else {
    localStorage.removeItem(`nightFilter_${currentClassId}`);
  }
  
  // Clear existing charts before reloading
  const chartIds = ['progress-chart', 'velocity-chart', 'day-of-week-chart', 'time-of-day-chart', 'grade-distribution-chart'];
  chartIds.forEach(id => {
    const canvas = document.getElementById(id);
    if (canvas) {
      const chart = Chart.getChart(canvas);
      if (chart) chart.destroy();
    }
  });
  
  // Reload all data with the filter
  loadSummary();
  loadBlockers();
  loadStudentHealth();
  loadStudentActivity();
  loadEngagementGaps();
  loadAssignmentTypes();
  loadGradeDistribution();
  loadVelocityChart();
  loadProgressChart();
  loadDayOfWeekChart();
  loadTimeOfDayChart();
  loadSectionProgress();
}

// Table sorting functionality
function initTableSorting() {
  const tables = ["difficulty-table", "risk-table", "activity-table"];

  tables.forEach((tableId) => {
    const table = document.getElementById(tableId);
    if (!table) return;

    const headers = table.querySelectorAll("th");
    headers.forEach((header, index) => {
      // Skip action column
      if (header.textContent.trim() === "Action") return;

      header.classList.add("sortable");
      header.dataset.column = index;
      header.dataset.order = "none";

      header.addEventListener("click", () => {
        sortTable(tableId, index, header);
      });
    });
  });
}

function sortTable(tableId, columnIndex, headerElement) {
  const table = document.getElementById(tableId);
  const tbody = table.querySelector("tbody");
  const rows = Array.from(tbody.querySelectorAll("tr"));

  // Store original order if not already stored
  if (!tbody.dataset.originalOrder) {
    tbody.dataset.originalOrder = JSON.stringify(rows.map((r) => r.outerHTML));
  }

  // Cycle through: none → asc → desc → none
  const currentOrder = headerElement.dataset.order || "none";
  let newOrder;
  if (currentOrder === "none") {
    newOrder = "asc";
  } else if (currentOrder === "asc") {
    newOrder = "desc";
  } else {
    newOrder = "none";
  }

  // Clear all sort indicators
  table.querySelectorAll("th").forEach((th) => {
    th.classList.remove("sort-asc", "sort-desc");
    th.dataset.order = "none";
  });

  // If returning to original order, restore it
  if (newOrder === "none") {
    const originalRows = JSON.parse(tbody.dataset.originalOrder);
    tbody.innerHTML = originalRows.join("");
    return;
  }

  // Set new sort indicator
  headerElement.dataset.order = newOrder;
  headerElement.classList.add(newOrder === "asc" ? "sort-asc" : "sort-desc");

  // Sort rows
  rows.sort((a, b) => {
    const aCell = a.cells[columnIndex];
    const bCell = b.cells[columnIndex];

    if (!aCell || !bCell) return 0;

    // Get text content, handling nested elements
    let aValue = aCell.textContent.trim();
    let bValue = bCell.textContent.trim();

    // Handle special cases for "Days Inactive" column
    if (aValue === "Today") aValue = "0 days";
    if (bValue === "Today") bValue = "0 days";
    if (aValue === "No activity") aValue = "999999 days";
    if (bValue === "No activity") bValue = "999999 days";

    // Try to parse as number (handles percentages, counts, etc)
    const aNum = parseFloat(aValue.replace(/[^0-9.-]/g, ""));
    const bNum = parseFloat(bValue.replace(/[^0-9.-]/g, ""));

    let comparison = 0;
    if (!isNaN(aNum) && !isNaN(bNum)) {
      comparison = aNum - bNum;
    } else {
      // For dates, try parsing
      const aDate = Date.parse(aValue);
      const bDate = Date.parse(bValue);
      if (!isNaN(aDate) && !isNaN(bDate)) {
        comparison = aDate - bDate;
      } else {
        comparison = aValue.localeCompare(bValue);
      }
    }

    return newOrder === "asc" ? comparison : -comparison;
  });

  // Re-append sorted rows
  rows.forEach((row) => tbody.appendChild(row));
}

// Initialize sorting for modal assignment table
function initModalTableSorting() {
  const table = document.getElementById("student-assignments-table");
  if (!table) return;

  const headers = table.querySelectorAll("th");
  headers.forEach((header, index) => {
    header.classList.add("sortable");
    header.dataset.column = index;
    header.dataset.order = "none";

    header.addEventListener("click", () => {
      sortTable("student-assignments-table", index, header);
    });
  });
}

// Initialize sorting for assignment modal student table
function initAssignmentModalTableSorting() {
  const table = document.getElementById("assignment-students-table");
  if (!table) return;

  const headers = table.querySelectorAll("th");
  headers.forEach((header, index) => {
    header.classList.add("sortable");
    header.dataset.column = index;
    header.dataset.order = "none";

    header.addEventListener("click", () => {
      sortTable("assignment-students-table", index, header);
    });
  });
}

// Initialize on page load
checkStoredClass();

// Auto-refresh every 60 seconds (only if dashboard is visible)
setInterval(() => {
  if (currentClassId) {
    init();
  }
}, 60000);


function closeSectionModal() {
  document.getElementById("section-modal").style.display = "none";
}

document.getElementById("section-modal").addEventListener("click", function(e) {
  if (e.target === this) {
    closeSectionModal();
  }
});

async function openSectionModal(sectionName) {
  const modal = document.getElementById("section-modal");
  modal.style.display = "flex";

  document.getElementById("modal-section-name").textContent = sectionName;
  document.getElementById("modal-section-info").textContent = "Loading...";

  // Fetch all data
  let students = await fetchData(`/classes/${currentClassId}/students`);
  const assignments = await fetchData(`/classes/${currentClassId}/assignments`);
  const progressions = await fetchData(`/classes/${currentClassId}/progressions`);

  if (!students || !assignments || !progressions) {
    document.getElementById("modal-section-info").textContent = "Failed to load data";
    return;
  }

  // Apply night filter if active
  const globalNightFilter = document.getElementById("global-night-filter")?.value;
  if (globalNightFilter) {
    students = students.filter(s => s.night && s.night.toLowerCase() === globalNightFilter.toLowerCase());
  }

  // Get assignments for this section
  const sectionAssignments = assignments.filter(a => a.section === sectionName);
  const assignmentIds = new Set(sectionAssignments.map(a => a.id));

  // Build student progress map
  const studentProgress = {};
  students.forEach(s => {
    studentProgress[s.id] = {
      student: s,
      completed: 0,
      total: sectionAssignments.length
    };
  });

  progressions.forEach(p => {
    if (assignmentIds.has(p.assignment_id) && studentProgress[p.student_id]) {
      if (p.grade && p.grade >= 0.7) {
        studentProgress[p.student_id].completed++;
      }
    }
  });

  // Categorize students
  const notStarted = [];
  const inProgress = [];

  Object.values(studentProgress).forEach(sp => {
    if (sp.completed === 0) {
      notStarted.push(sp);
    } else if (sp.completed < sp.total) {
      inProgress.push(sp);
    }
  });

  document.getElementById("modal-section-info").textContent = 
    `${sectionAssignments.length} assignments | ${notStarted.length} not started | ${inProgress.length} in progress`;

  // Render not started
  const notStartedBody = document.getElementById("section-not-started-body");
  if (notStarted.length === 0) {
    notStartedBody.innerHTML = '<tr><td colspan="2">All students have started this section</td></tr>';
  } else {
    notStartedBody.innerHTML = notStarted
      .map((sp, index) => `
        <tr style="cursor:pointer;" onclick="previousSectionModal = '${escapeForAttr(sectionName)}'; closeSectionModal(); openStudentModal('${sp.student.id}')">
          <td>${maskName(sp.student.first_name, sp.student.last_name, index)}</td>
          <td>${sp.student.night || '-'}</td>
        </tr>
      `)
      .join("");
  }

  // Render in progress
  const inProgressBody = document.getElementById("section-in-progress-body");
  if (inProgress.length === 0) {
    inProgressBody.innerHTML = '<tr><td colspan="3">No students in progress</td></tr>';
  } else {
    inProgressBody.innerHTML = inProgress
      .map((sp, index) => {
        const progressPct = (sp.completed / sp.total) * 100;
        return `
          <tr style="cursor:pointer;" onclick="previousSectionModal = '${escapeForAttr(sectionName)}'; closeSectionModal(); openStudentModal('${sp.student.id}')">
            <td>${maskName(sp.student.first_name, sp.student.last_name, index)}</td>
            <td>${sp.student.night || '-'}</td>
            <td>
              <div style="display:flex; align-items:center; gap:10px;">
                <div class="progress-bar" style="width:80px;">
                  <div class="fill" style="width:${progressPct}%"></div>
                </div>
                <span>${sp.completed}/${sp.total} (${Math.round(progressPct)}%)</span>
              </div>
            </td>
          </tr>
        `;
      })
      .join("");
  }
}
