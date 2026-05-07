import { z } from "zod";

export const shortText = z.string().trim().min(1).max(500);
export const mediumText = z.string().trim().min(1).max(4_000);
export const longText = z.string().trim().min(1).max(100_000);
export const pathText = z.string().trim().min(1).max(1_000);
export const idText = z.string().trim().min(1).max(200);

export function optionalText(schema = mediumText) {
  return schema.optional();
}

export function boundedArray<T extends z.ZodTypeAny>(schema: T, max = 100) {
  return z.array(schema).max(max);
}

