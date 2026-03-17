import { useState, useEffect } from "react";

export interface MousePosition {
  x: number;
  y: number;
}

/**
 * 追踪鼠标在窗口中的位置。
 * - `position`：当前鼠标坐标（clientX / clientY）
 * - `inWindow`：鼠标是否在窗口内（用于离开时重置视线）
 */
export function useMousePosition() {
  const [position, setPosition] = useState<MousePosition>({ x: 0, y: 0 });
  const [inWindow, setInWindow] = useState(true);

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      setPosition({ x: e.clientX, y: e.clientY });
      setInWindow(true);
    };

    const handleMouseLeave = () => {
      setInWindow(false);
    };

    window.addEventListener("mousemove", handleMouseMove);
    window.addEventListener("mouseleave", handleMouseLeave);

    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseleave", handleMouseLeave);
    };
  }, []);

  return { position, inWindow };
}
