/** 宠物所有可用的情绪状态 */
export type PetMood = "idle" | "happy" | "sleepy" | "excited" | "curious";

/**
 * 宠物形象抽象接口。
 *
 * 通过此接口将"形象实现"与"交互逻辑"解耦：
 * - 更换 SVG：修改 petAppearance.ts，其他逻辑不动
 * - 接入 Live2D：实现同一接口，替换 Pet.tsx 内部渲染即可
 * - 接入序列帧：同上
 */
export interface PetAppearance {
  /** 情绪状态到 CSS 表情类名的映射 */
  expressions: Record<PetMood, string>;

  /** 眼睛容器的 CSS 选择器（用于 Step 2 视线追踪 transform） */
  eyesSelector: string;

  /** 头部容器的 CSS 选择器（用于 Step 2 头部跟随 transform） */
  headSelector: string;

  /** 宠物根容器的 CSS 类名（情绪修饰符将附加在此基础上） */
  containerClass: string;
}

/** 当前 SVG 形象的默认配置 */
export const defaultPetAppearance: PetAppearance = {
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
