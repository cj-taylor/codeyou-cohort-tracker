// API functions for fetching data from the backend

import { state } from './state.js';

/**
 * Generic fetch wrapper with error handling
 * @param {string} endpoint - API endpoint (e.g., "/classes")
 * @returns {Promise<any>} JSON response or null on error
 */
export async function fetchData(endpoint) {
  try {
    const response = await fetch(`${state.API_BASE}${endpoint}`);
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return await response.json();
  } catch (error) {
    console.error(`Error fetching ${endpoint}:`, error);
    return null;
  }
}

/**
 * Get all classes (active or all)
 * @param {boolean} showAll - Include inactive classes
 * @returns {Promise<Array>} List of classes
 */
export async function getClasses(showAll = false) {
  const endpoint = showAll ? '/classes?all=true' : '/classes';
  return fetchData(endpoint);
}

/**
 * Get students for a class
 * @param {string} classId - Class ID
 * @returns {Promise<Array>} List of students
 */
export async function getStudents(classId) {
  return fetchData(`/classes/${classId}/students`);
}

/**
 * Get assignments for a class
 * @param {string} classId - Class ID
 * @returns {Promise<Array>} List of assignments
 */
export async function getAssignments(classId) {
  return fetchData(`/classes/${classId}/assignments`);
}

/**
 * Get progressions for a class
 * @param {string} classId - Class ID
 * @returns {Promise<Array>} List of progressions
 */
export async function getProgressions(classId) {
  return fetchData(`/classes/${classId}/progressions`);
}

/**
 * Get student activity data
 * @param {string} classId - Class ID
 * @param {string} night - Optional night filter
 * @returns {Promise<Array>} Student activity data
 */
export async function getStudentActivity(classId, night = null) {
  const endpoint = night
    ? `/classes/${classId}/metrics/student-activity?night=${night}`
    : `/classes/${classId}/metrics/student-activity`;
  return fetchData(endpoint);
}

/**
 * Activate a class
 * @param {string} classId - Class ID
 * @returns {Promise<any>} Response
 */
export async function activateClass(classId) {
  const response = await fetch(`${state.API_BASE}/classes/${classId}/activate`, {
    method: 'POST'
  });
  if (!response.ok) throw new Error('Failed to activate');
  return response.json();
}

/**
 * Deactivate a class
 * @param {string} classId - Class ID
 * @returns {Promise<any>} Response
 */
export async function deactivateClass(classId) {
  const response = await fetch(`${state.API_BASE}/classes/${classId}/deactivate`, {
    method: 'POST'
  });
  if (!response.ok) throw new Error('Failed to deactivate');
  return response.json();
}
