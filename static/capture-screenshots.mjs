import { chromium } from 'playwright';

// Helper to add highlight box around element
async function highlightElement(page, selector) {
  await page.evaluate((sel) => {
    const el = document.querySelector(sel);
    if (!el) return;
    
    const rect = el.getBoundingClientRect();
    const highlight = document.createElement('div');
    highlight.id = 'screenshot-highlight';
    highlight.style.cssText = `
      position: fixed;
      left: ${rect.left - 8}px;
      top: ${rect.top - 8}px;
      width: ${rect.width + 16}px;
      height: ${rect.height + 16}px;
      border: 4px solid #FF6B35;
      border-radius: 8px;
      pointer-events: none;
      z-index: 999999;
      box-shadow: 0 0 0 4px rgba(255, 107, 53, 0.3);
    `;
    document.body.appendChild(highlight);
  }, selector);
}

async function removeHighlight(page) {
  await page.evaluate(() => {
    const highlight = document.getElementById('screenshot-highlight');
    if (highlight) highlight.remove();
  });
}

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  await page.setViewportSize({ width: 1920, height: 1080 });

  console.log('Capturing screenshots for dashboard guide...\n');

  // Step 1: Class selection screen
  console.log('1/12: Class selection screen...');
  await page.goto('http://localhost:3000');
  await page.waitForLoadState('networkidle');
  await page.waitForSelector('.class-grid .class-card', { timeout: 5000 });
  
  // Ensure demo mode is active
  await page.evaluate(() => {
    if (window.demoMode === false) {
      const checkbox = document.getElementById('demo-mode');
      if (checkbox) {
        checkbox.checked = true;
        window.toggleDemoMode();
      }
    }
  });
  await page.waitForTimeout(500);
  
  await highlightElement(page, '.class-grid');
  await page.screenshot({ path: '../docs/screenshots/01-class-selection.png', fullPage: true });
  await removeHighlight(page);

  // Step 2: Click second class card (Class 1 has engagement gaps)
  console.log('2/12: Selecting a class...');
  await page.click('.class-card:nth-child(2)');
  await page.waitForSelector('#dashboard', { state: 'visible' });
  await page.waitForLoadState('networkidle');
  
  // Wait for engagement gaps API call to complete
  await page.waitForResponse(
    response => response.url().includes('/metrics/engagement-gaps') && response.status() === 200,
    { timeout: 10000 }
  ).catch(() => console.log('   No engagement gaps data or timeout'));
  
  await page.waitForTimeout(1000); // Extra time for rendering
  
  // Check if engagement gaps are visible
  const gapsVisible = await page.evaluate(() => {
    const alert = document.getElementById('engagement-gap-alert');
    return alert && window.getComputedStyle(alert).display !== 'none';
  });
  console.log(`   Engagement gaps visible: ${gapsVisible}`);
  
  if (!gapsVisible) {
    console.log('   Note: Engagement gaps require students with 7-14 days inactivity in test data');
  }
  
  await highlightElement(page, '#dashboard');
  await page.screenshot({ path: '../docs/screenshots/02-dashboard-overview.png', fullPage: true });
  await removeHighlight(page);

  // Step 3: Night filter dropdown
  console.log('3/12: Night filter dropdown...');
  const nightFilterVisible = await page.isVisible('#global-night-filter');
  if (nightFilterVisible) {
    await page.click('#global-night-filter');
    await page.waitForTimeout(500);
    await highlightElement(page, '#global-night-filter');
    await page.screenshot({ path: '../docs/screenshots/03-night-filter.png' });
    await removeHighlight(page);

    // Step 4: Select a night (Thursday)
    console.log('4/12: Night filtered...');
    const nightOptions = await page.$$('#global-night-filter option');
    if (nightOptions.length > 1) {
      // Try to find Thursday, otherwise use index 1
      let selectedValue = null;
      for (let i = 0; i < nightOptions.length; i++) {
        const text = await nightOptions[i].textContent();
        if (text.toLowerCase().includes('thursday') || text.toLowerCase().includes('thurs')) {
          const value = await nightOptions[i].getAttribute('value');
          selectedValue = value;
          console.log(`   Found Thursday with value: ${value}`);
          break;
        }
      }
      
      if (selectedValue) {
        await page.selectOption('#global-night-filter', selectedValue);
      } else {
        await page.selectOption('#global-night-filter', { index: 1 });
        console.log('   Selected: index 1 (Thursday not found)');
      }
      
      // Manually trigger the onchange event and call the filter function
      await page.evaluate(() => {
        if (window.applyNightFilter) {
          window.applyNightFilter();
        }
      });
      
      await page.waitForLoadState('networkidle');
      
      // Wait for engagement gaps API call after night filter
      await page.waitForResponse(
        response => response.url().includes('/metrics/engagement-gaps') && response.status() === 200,
        { timeout: 10000 }
      ).catch(() => console.log('   No engagement gaps data after night filter'));
      
      await page.waitForTimeout(1000); // Extra time for rendering
      await highlightElement(page, '#global-night-filter');
      await page.screenshot({ path: '../docs/screenshots/04-night-filtered.png', fullPage: true });
      await removeHighlight(page);
      
      // Step 4c: Highlight an engagement gap student before clicking
      console.log('4c/18: Engagement gap student highlight...');
      const gapStudent = await page.$('#engagement-gap-list > div:first-child');
      if (gapStudent) {
        await page.evaluate(() => {
          window.scrollTo(0, 0);
        });
        await page.waitForTimeout(300);
        await highlightElement(page, '#engagement-gap-list > div:first-child');
        await page.screenshot({ path: '../docs/screenshots/09a-engagement-gap-student-highlight.png' });
        await removeHighlight(page);
        
        // Click to open student modal
        await page.click('#engagement-gap-list > div:first-child');
        await page.waitForSelector('#student-modal', { state: 'visible', timeout: 2000 });
        await page.waitForTimeout(1000);
        await highlightElement(page, '#student-modal .modal-content');
        await page.screenshot({ path: '../docs/screenshots/09b-engagement-gap-student-modal.png' });
        await removeHighlight(page);
        
        // Close modal
        await page.evaluate(() => window.closeStudentModal());
        await page.waitForTimeout(500);
      }
      
      // Step 4b: Capture engagement gaps area (top of page after night filter)
      console.log('4b/12: Engagement gaps area...');
      
      await page.evaluate(() => {
        window.scrollTo(0, 0);
      });
      await page.waitForTimeout(500);
      await highlightElement(page, '#engagement-gap-alert');
      await page.screenshot({ path: '../docs/screenshots/09-engagement-gaps.png' });
      await removeHighlight(page);
    }
  } else {
    console.log('3-4/12: Night filter not available (no night data)');
  }

  // Step 5: Assignment Types
  console.log('5/12: Assignment types chart...');
  await page.evaluate(() => {
    document.querySelector('#assignment-type-table')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#assignment-type-table');
  await page.screenshot({ path: '../docs/screenshots/05-assignment-types.png' });
  await removeHighlight(page);

  // Step 6: Grade Distribution
  console.log('6/12: Grade distribution...');
  await page.evaluate(() => {
    document.querySelector('#grade-distribution-chart')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#grade-distribution-chart');
  await page.screenshot({ path: '../docs/screenshots/06-grade-distribution.png' });
  await removeHighlight(page);

  // Step 7: Velocity Chart
  console.log('7/12: Velocity chart...');
  await page.evaluate(() => {
    document.querySelector('#velocity-chart')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#velocity-chart');
  await page.screenshot({ path: '../docs/screenshots/07-velocity-chart.png' });
  await removeHighlight(page);

  // Step 8: Students at Risk
  console.log('8/12: Students at risk table...');
  await page.evaluate(() => {
    document.querySelector('#risk-table')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#risk-table');
  await page.screenshot({ path: '../docs/screenshots/08-at-risk-table.png' });
  await removeHighlight(page);

  // Step 9: Engagement Gaps (already captured after night filter in step 4b)
  console.log('9/12: Engagement gaps (skipped - captured in step 4b)...');

  // Step 10: Student Activity table
  console.log('10/24: Student activity table...');
  await page.evaluate(() => {
    document.querySelector('#activity-table')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#activity-table');
  await page.screenshot({ path: '../docs/screenshots/10a-student-activity.png' });
  await removeHighlight(page);

  // Step 10b: Student search functionality
  console.log('10b/24: Student search...');
  await page.type('#student-search', 'Student');
  await page.waitForTimeout(500);
  await highlightElement(page, '#student-search');
  await page.screenshot({ path: '../docs/screenshots/10b-student-search.png' });
  await removeHighlight(page);
  await page.fill('#student-search', ''); // Clear search

  // Step 10c: Performance by Night
  console.log('10c/24: Performance by night...');
  await page.evaluate(() => {
    document.querySelector('#night-summary-container')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#night-summary-container');
  await page.screenshot({ path: '../docs/screenshots/10c-performance-by-night.png' });
  await removeHighlight(page);

  // Step 10d: Progress Over Time
  console.log('10d/24: Progress over time...');
  await page.evaluate(() => {
    document.querySelector('#progress-chart')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#progress-chart');
  await page.screenshot({ path: '../docs/screenshots/10d-progress-over-time.png' });
  await removeHighlight(page);

  // Step 10e: Activity by Day of Week
  console.log('10e/24: Activity by day of week...');
  await page.evaluate(() => {
    document.querySelector('#day-of-week-chart')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#day-of-week-chart');
  await page.screenshot({ path: '../docs/screenshots/10e-activity-by-day.png' });
  await removeHighlight(page);

  // Step 10f: Activity by Time of Day
  console.log('10f/24: Activity by time of day...');
  await page.evaluate(() => {
    document.querySelector('#time-of-day-chart')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#time-of-day-chart');
  await page.screenshot({ path: '../docs/screenshots/10f-activity-by-time.png' });
  await removeHighlight(page);

  // Step 10g: Progress by Section
  console.log('10g/24: Progress by section...');
  await page.evaluate(() => {
    document.querySelector('#section-progress-table')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#section-progress-table');
  await page.screenshot({ path: '../docs/screenshots/10g-progress-by-section.png' });
  await removeHighlight(page);

  // Step 10: Assignment Difficulty
  console.log('10/24: Assignment difficulty...');
  await page.evaluate(() => {
    document.querySelector('#difficulty-table')?.scrollIntoView({ behavior: 'instant', block: 'center' });
  });
  await page.waitForTimeout(500);
  await highlightElement(page, '#difficulty-table');
  await page.screenshot({ path: '../docs/screenshots/10-assignment-difficulty.png' });
  await removeHighlight(page);

  // Step 11: Table sorted
  console.log('11/12: Table sorting...');
  await page.click('#difficulty-table th:nth-child(3)');
  await page.waitForTimeout(500);
  await highlightElement(page, '#difficulty-table th:nth-child(3)');
  await page.screenshot({ path: '../docs/screenshots/11-table-sorted.png' });
  await removeHighlight(page);

  // Step 11b: Click on a student row to show modal
  console.log('11b/24: Student detail modal...');
  const studentRow = await page.$('#risk-table tbody tr:first-child');
  if (studentRow) {
    await page.evaluate(() => {
      document.querySelector('#risk-table')?.scrollIntoView({ behavior: 'instant', block: 'center' });
    });
    await page.waitForTimeout(500);
    await highlightElement(page, '#risk-table tbody tr:first-child');
    await page.waitForTimeout(300);
    await page.screenshot({ path: '../docs/screenshots/11b-student-row-highlight.png' });
    await removeHighlight(page);
    
    await page.click('#risk-table tbody tr:first-child');
    await page.waitForSelector('#student-modal', { state: 'visible', timeout: 2000 });
    await page.waitForTimeout(1000);
    await highlightElement(page, '#student-modal .modal-content');
    await page.screenshot({ path: '../docs/screenshots/11c-student-modal.png' });
    await removeHighlight(page);
    
    // Close modal by calling the close function
    await page.evaluate(() => window.closeStudentModal());
    await page.waitForTimeout(500);
  }

  // Step 11d: Click on an assignment row to show modal
  console.log('11d/24: Assignment detail modal...');
  const assignmentRow = await page.$('#difficulty-table tbody tr:first-child');
  if (assignmentRow) {
    await page.evaluate(() => {
      document.querySelector('#difficulty-table')?.scrollIntoView({ behavior: 'instant', block: 'center' });
    });
    await page.waitForTimeout(500);
    await highlightElement(page, '#difficulty-table tbody tr:first-child');
    await page.waitForTimeout(300);
    await page.screenshot({ path: '../docs/screenshots/11d-assignment-row-highlight.png' });
    await removeHighlight(page);
    
    await page.click('#difficulty-table tbody tr:first-child');
    await page.waitForSelector('#assignment-modal', { state: 'visible', timeout: 2000 });
    await page.waitForTimeout(1000);
    await highlightElement(page, '#assignment-modal .modal-content');
    await page.screenshot({ path: '../docs/screenshots/11e-assignment-modal.png' });
    await removeHighlight(page);
    
    // Close modal by calling the close function
    await page.evaluate(() => window.closeAssignmentModal());
    await page.waitForTimeout(500);
  }

  // Step 12: Back to class selection
  console.log('12/24: Back button...');
  await highlightElement(page, '.back-button');
  await page.waitForTimeout(300);
  await page.screenshot({ path: '../docs/screenshots/12a-back-button.png' });
  await removeHighlight(page);
  
  await page.click('.back-button');
  await page.waitForSelector('#class-selection', { state: 'visible' });
  
  // Ensure demo mode is still active
  await page.evaluate(() => {
    if (window.demoMode === false) {
      const checkbox = document.getElementById('demo-mode');
      if (checkbox) {
        checkbox.checked = true;
        window.toggleDemoMode();
      }
    }
  });
  await page.waitForTimeout(500);
  
  await highlightElement(page, '.class-grid');
  await page.screenshot({ path: '../docs/screenshots/12-back-to-classes.png', fullPage: true });
  await removeHighlight(page);

  await browser.close();
  console.log('\n✓ All screenshots captured successfully!');
  console.log('Screenshots saved to docs/screenshots/');
})();
