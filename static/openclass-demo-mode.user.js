// ==UserScript==
// @name         OpenClass Demo Mode
// @namespace    http://tampermonkey.net/
// @version      1.0
// @description  Toggle student name visibility in OpenClass UI
// @author       You
// @match        https://classroom.code-you.org/*
// @grant        none
// ==/UserScript==

(function() {
    'use strict';

    // Mask name similar to the cohort tracker implementation
    function maskName(name) {
        if (!name || typeof name !== 'string') return name;
        const parts = name.split(',').map(p => p.trim());
        if (parts.length === 2) {
            // "Last, First" format
            return `Student ${parts[1].charAt(0)}${parts[0].charAt(0)}`;
        }
        // "First Last" or single name
        const words = name.trim().split(/\s+/);
        if (words.length >= 2) {
            return `Student ${words[0].charAt(0)}${words[words.length - 1].charAt(0)}`;
        }
        return `Student ${words[0].charAt(0)}`;
    }

    let checkbox;
    const maskedElements = new WeakSet();

    function init() {
        // Create floating checkbox
        const container = document.createElement('div');
        container.style.cssText = 'position: fixed; top: 100px; right: 10px; z-index: 10000; background: white; padding: 10px; border: 1px solid #ccc; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.2);';
        
        const label = document.createElement('label');
        label.style.cssText = 'display: flex; align-items: center; gap: 8px; cursor: pointer; font-family: sans-serif; font-size: 14px;';
        
        checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.id = 'demo-mode-toggle';
        
        // Load saved state
        const savedState = localStorage.getItem('openclass-demo-mode');
        checkbox.checked = savedState === 'true';
        
        label.appendChild(checkbox);
        label.appendChild(document.createTextNode('Demo Mode'));
        container.appendChild(label);
        document.body.appendChild(container);

        // Save state on change
        checkbox.addEventListener('change', () => {
            localStorage.setItem('openclass-demo-mode', checkbox.checked);
        });

        // Run on short polling interval
        setInterval(maskStudentNames, 100);

        // Also run on DOM changes
        const observer = new MutationObserver(maskStudentNames);
        observer.observe(document.body, { childList: true, subtree: true });
    }

    function maskStudentNames() {
        if (!checkbox || !checkbox.checked) return;

        // Progressions page - student list links
        document.querySelectorAll('.class-page-grades-table-name-profile-link').forEach(link => {
            if (!maskedElements.has(link)) {
                const originalName = link.textContent.trim();
                link.textContent = maskName(originalName);
                maskedElements.add(link);
            }
        });

        // Lesson dashboard - student items
        document.querySelectorAll('.lesson-metrics-student-item').forEach(item => {
            const nameSpan = item.querySelector('.lesson-metrics-student-item-section-name span');
            if (nameSpan && !maskedElements.has(nameSpan)) {
                const originalName = nameSpan.textContent.trim();
                nameSpan.textContent = maskName(originalName);
                maskedElements.add(nameSpan);
            }
        });
    }

    // Wait for page to load
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
