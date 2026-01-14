// Utility functions for formatting and masking data

/**
 * Mask student name for demo mode
 * @param {string} firstName - Student's first name
 * @param {string} lastName - Student's last name
 * @param {number} index - Student index for numbering
 * @param {boolean} demoMode - Whether demo mode is enabled
 * @returns {string} Masked or actual name
 */
export function maskName(firstName, lastName, index, demoMode) {
  if (!demoMode) return `${firstName} ${lastName}`;
  return `Student ${index + 1}`;
}

/**
 * Mask email address for demo mode
 * @param {string} email - Student's email
 * @param {number} index - Student index for numbering
 * @param {boolean} demoMode - Whether demo mode is enabled
 * @returns {string} Masked or actual email
 */
export function maskEmail(email, index, demoMode) {
  if (!demoMode) return email;
  return `student${index + 1}@example.com`;
}

/**
 * Format decimal as percentage
 * @param {number} value - Decimal value (0-1)
 * @returns {string} Formatted percentage (e.g., "75.5%")
 */
export function formatPercent(value) {
  return (value * 100).toFixed(1) + '%';
}

/**
 * Format Unix timestamp as readable date
 * @param {number} timestamp - Unix timestamp
 * @returns {string} Formatted date string
 */
export function formatDate(timestamp) {
  if (!timestamp) return 'Never';
  const date = new Date(timestamp * 1000);
  return date.toLocaleString();
}

/**
 * Convert ISO week (YYYY-WW) to date range string
 * @param {string} isoWeek - ISO week format (e.g., "2025-35")
 * @returns {string} Date range (e.g., "Aug 25 - Aug 31")
 */
export function getWeekDateRange(isoWeek) {
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
  
  const fmt = (d) => d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
  return `${fmt(weekStart)} - ${fmt(weekEnd)}`;
}
