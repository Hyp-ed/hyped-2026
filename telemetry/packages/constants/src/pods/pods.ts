import {
  type Pod,
  type RawPod,
  PodSchema,
  RawPodSchema,
} from '@hyped/telemetry-types';
import * as YAML from 'yaml';
import { podsYamlContent } from './pods-data.generated.js';

const yamlData = YAML.parse(podsYamlContent);

// First, parse the raw YAML data for each pod into the RawPod format,
// which doesn't include the 'id' fields in measurements and statuses
const parsedData: Record<string, RawPod> = Object.fromEntries(
  Object.entries(yamlData.pods).map(([podId, podData]) => {
    return [podId, RawPodSchema.parse(podData)];
  }),
);

// Now, transform the parsed data into the final Pod format, adding the 'id' fields
export const pods: Record<string, Pod> = Object.fromEntries(
  Object.entries(parsedData).map(([podId, podData]) => {
    const measurementsWithId = Object.fromEntries(
      Object.entries(podData.measurements).map(
        ([measurementId, measurement]) => [
          measurementId,
          { ...measurement, id: measurementId },
        ],
      ),
    );
    const statusesWithId = Object.fromEntries(
      Object.entries(podData.statuses).map(([statusId, status]) => [
        statusId,
        { ...status, id: statusId },
      ]),
    );
    return [
      podId,
      PodSchema.parse({
        ...podData,
        id: podId,
        measurements: measurementsWithId,
        statuses: statusesWithId,
      }),
    ];
  }),
);

export const podIds = Object.keys(pods);
