import { z } from 'zod';

export const LimitSchema = z.object({
  low: z.number(),
  high: z.number(),
});

export type Limit = z.infer<typeof LimitSchema>;

export const MeasurementLimitsSchema = z.object({
  critical: LimitSchema,
  warning: LimitSchema.optional(),
});

export type MeasurementLimits = z.infer<typeof MeasurementLimitsSchema>;

export const MeasurementSchemaNoId = z.object({
  label: z.string(),
  unit: z.string(),
  kind: z.string(),
  format: z.enum(['float', 'integer']),
  limits: MeasurementLimitsSchema,
});
export const MeasurementSchema = MeasurementSchemaNoId.extend({
  id: z.string(),
});

export type Measurement = z.infer<typeof MeasurementSchema>;

export const StatusSchemaNoId = z.object({
  label: z.string(),
  kind: z.string(),
  format: z.literal('enum'),
  values: z.array(
    z.object({
      value: z.number(),
      label: z.string(),
    }),
  ),
});
export const StatusSchema = StatusSchemaNoId.extend({
  id: z.string(),
});

export type Status = z.infer<typeof StatusSchema>;

export const BasePodSchema = z.object({
  label: z.string(),
  mode: z.enum(['ALL_SYSTEMS_ON', 'LEVITATION_ONLY', 'LIM_ONLY']),
});

export const PodSchema = BasePodSchema.extend({
  id: z.string(),
  measurements: z.record(MeasurementSchema),
  statuses: z.record(StatusSchema),
});

export const RawPodSchema = BasePodSchema.extend({
  measurements: z.record(MeasurementSchemaNoId),
  statuses: z.record(StatusSchemaNoId),
});

export type Pod = z.infer<typeof PodSchema>;
export type RawPod = z.infer<typeof RawPodSchema>;
