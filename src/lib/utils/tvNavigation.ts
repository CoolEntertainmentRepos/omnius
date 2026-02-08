// TV Spatial Navigation using spatial-navigation-js
// https://www.npmjs.com/package/spatial-navigation-js

let SpatialNavigation: any = null;
let initialized = false;

export async function initSpatialNavigation() {
  console.log('[SpatialNav] initSpatialNavigation called, initialized:', initialized);

  if (initialized) {
    console.log('[SpatialNav] Already initialized, skipping');
    return;
  }

  if (typeof window === 'undefined') {
    console.log('[SpatialNav] No window, skipping (SSR)');
    return;
  }

  try {
    console.log('[SpatialNav] Importing spatial-navigation-js...');
    // Dynamic import for SSR compatibility
    const module = await import('spatial-navigation-js');
    console.log('[SpatialNav] Module loaded:', module);
    SpatialNavigation = module.default;
    console.log('[SpatialNav] SpatialNavigation:', SpatialNavigation);

    if (!SpatialNavigation || typeof SpatialNavigation.init !== 'function') {
      console.error('[SpatialNav] Invalid module - no init function');
      return;
    }

    SpatialNavigation.init();
    console.log('[SpatialNav] init() called');

    // Add default section with all focusable elements
    SpatialNavigation.add({
      id: 'default',
      selector: 'button:not([disabled]), a[href], input:not([disabled]), [tabindex="0"], .focusable',
      defaultElement: '[data-default-focus]',
      straightOnly: false,
      straightOverlapThreshold: 0.35,
      rememberSource: true,
      enterTo: 'default-element',
      restrict: 'none', // Allow navigation to any element on page
    });
    console.log('[SpatialNav] Section added');

    SpatialNavigation.makeFocusable();
    console.log('[SpatialNav] makeFocusable() called');

    // Listen for focus events to scroll into view
    document.addEventListener('sn:focused', (e: any) => {
      const el = e.target as HTMLElement;
      console.log('[SpatialNav] Focus event:', el?.tagName, el?.className);
      if (el && el.scrollIntoView) {
        el.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
      }
    });

    // Throttle held-down key repeats to prevent janky scrolling
    let lastRepeatTime = 0;
    const REPEAT_THROTTLE = 150; // ms — allows ~6 moves/sec max
    document.addEventListener('keydown', (e) => {
      if (e.repeat) {
        const now = Date.now();
        if (now - lastRepeatTime < REPEAT_THROTTLE) {
          e.preventDefault();
          e.stopImmediatePropagation();
          return;
        }
        lastRepeatTime = now;
      }
    }, true); // capture phase — runs before spatial-nav and component handlers

    initialized = true;
    console.log('[SpatialNav] Initialization complete!');
  } catch (error) {
    console.error('[SpatialNav] Error initializing:', error);
  }
}

export function addSection(id: string, selector: string, options?: {
  defaultElement?: string;
  straightOnly?: boolean;
  restrict?: 'self-first' | 'self-only' | 'none';
  leaveFor?: { up?: string; down?: string; left?: string; right?: string };
}) {
  if (!SpatialNavigation) return;

  // Remove existing section first to avoid "already existed" errors (e.g. HMR or re-mount)
  try {
    SpatialNavigation.remove(id);
  } catch {
    // Section didn't exist, that's fine
  }

  SpatialNavigation.add({
    id,
    selector,
    defaultElement: options?.defaultElement,
    straightOnly: options?.straightOnly ?? false,
    restrict: options?.restrict || 'self-first',
    leaveFor: options?.leaveFor,
  });
  SpatialNavigation.makeFocusable();
}

export function removeSection(id: string) {
  if (SpatialNavigation) {
    SpatialNavigation.remove(id);
  }
}

export function setFocus(elementOrSelector: string | HTMLElement): void {
  if (!SpatialNavigation) return;

  if (typeof elementOrSelector === 'string') {
    SpatialNavigation.focus(elementOrSelector);
  } else {
    SpatialNavigation.focus(elementOrSelector);
  }
}

export function navigate(direction: 'up' | 'down' | 'left' | 'right'): boolean {
  if (!SpatialNavigation) return false;
  SpatialNavigation.move(direction);
  return true;
}

export function pause() {
  if (SpatialNavigation) {
    SpatialNavigation.pause();
  }
}

export function resume() {
  if (SpatialNavigation) {
    SpatialNavigation.resume();
  }
}

export function makeFocusable() {
  if (SpatialNavigation) {
    SpatialNavigation.makeFocusable();
  }
}

export function setInitialFocus(selector: string, delay = 100): void {
  setTimeout(() => {
    if (SpatialNavigation) {
      SpatialNavigation.makeFocusable();
      SpatialNavigation.focus(selector);
    }
  }, delay);
}
