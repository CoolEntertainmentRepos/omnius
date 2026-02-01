declare module 'spatial-navigation-js' {
  interface SpatialNavigationOptions {
    id?: string;
    selector?: string;
    defaultElement?: string;
    straightOnly?: boolean;
    straightOverlapThreshold?: number;
    rememberSource?: boolean;
    enterTo?: 'default-element' | 'last-focused' | '';
    restrict?: 'self-first' | 'self-only' | 'none';
    leaveFor?: {
      up?: string;
      down?: string;
      left?: string;
      right?: string;
    };
  }

  interface SpatialNavigation {
    init(): void;
    uninit(): void;
    clear(): void;
    add(options: SpatialNavigationOptions): void;
    remove(id: string): void;
    disable(id?: string): void;
    enable(id?: string): void;
    pause(): void;
    resume(): void;
    focus(elementOrSelector?: string | HTMLElement, silent?: boolean): boolean;
    move(direction: 'up' | 'down' | 'left' | 'right', selector?: string): boolean;
    makeFocusable(id?: string): void;
  }

  const SpatialNavigation: SpatialNavigation;
  export default SpatialNavigation;
}
