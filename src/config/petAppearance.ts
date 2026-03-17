/**
 * 宠物形象配置文件
 *
 * ── 如何替换形象 ──────────────────────────────────────────────────
 *
 * 当前：手绘 SVG 内联于 Pet.tsx
 *
 * 替换为精细 SVG：
 *   1. 将新 SVG 引入 Pet.tsx，保持相同的 className 结构
 *      (.pet-head / .pet-eyes / .expression-* 分组)
 *   2. 不需要修改此配置文件
 *
 * 替换为 Live2D：
 *   1. 安装 pixi-live2d-display
 *   2. 创建 Live2DPet.tsx，实现相同的 containerClass + 表情切换逻辑
 *   3. 导出一个新的 PetAppearance 对象（live2dAppearance）
 *   4. 在 App.tsx 中将 appearance={live2dAppearance} 传给对应组件
 *
 * 替换为序列帧精灵图：
 *   1. 用 canvas / <img> + sprite sheet 替换 SVG
 *   2. 将 expressions 映射到精灵图的帧范围
 *   3. 同样导出一个新的 PetAppearance 对象
 *
 * ──────────────────────────────────────────────────────────────────
 */

import type { PetAppearance } from "../types/pet";

/** 当前使用的 SVG 手绘形象配置 */
export const svgAppearance: PetAppearance = {
  expressions: {
    idle:    "expression-idle",
    happy:   "expression-happy",
    sleepy:  "expression-sleepy",
    excited: "expression-excited",
    curious: "expression-curious",
  },
  eyesSelector:   ".pet-eyes",
  headSelector:   ".pet-head",
  containerClass: "pet-container",
};

// ── 未来形象占位（取消注释并实现即可切换）────────────────────────────

// export const live2dAppearance: PetAppearance = {
//   expressions: { idle: 'idle', happy: 'happy', ... },
//   eyesSelector: '.live2d-eyes',
//   headSelector: '.live2d-head',
//   containerClass: 'pet-container',
// };

// export const spriteAppearance: PetAppearance = {
//   expressions: { idle: 'frame-0-3', happy: 'frame-4-7', ... },
//   eyesSelector: '.sprite-eyes',
//   headSelector: '.sprite-head',
//   containerClass: 'pet-container',
// };
