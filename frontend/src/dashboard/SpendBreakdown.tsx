import { DonutChart } from "@tremor/react";
import { SpendPerMonth } from "../bindings/SpendPerMonth";
import { valueFormatter } from "../lib/numbers";

export function SpendBreakdown({
  spendPerMonth: spend_per_month,
  presets,
}: {
  spendPerMonth: SpendPerMonth;
  presets: Preset[];
}) {
  const data = [] as { name: string; value: number }[];

  if (presets.length === 0) {
    for (const y of Object.values(spend_per_month.months)) {
      for (const c of Object.values(y)) {
        for (const [cat, amount] of Object.entries(c)) {
          const elementIndex = data.findIndex((d) => d.name == cat);
          if (elementIndex != -1) {
            data[elementIndex].value += amount;
          } else {
            data.push({ name: cat, value: amount });
          }
        }
      }
    }
  } else {
    for (const preset of presets) {
      if (
        spend_per_month.months[preset.month] &&
        spend_per_month.months[preset.month][preset.year]
      ) {
        const category = spend_per_month.months[preset.month][preset.year];

        for (const [cat, amount] of Object.entries(category)) {
          const elementIndex = data.findIndex((d) => d.name == cat);
          if (elementIndex != -1) {
            data[elementIndex].value + amount;
          } else {
            data.push({ name: cat, value: amount });
          }
        }
      }
    }
  }

  return (
    <>
      <DonutChart
        className="h-72 mt-4"
        data={data}
        index="name"
        valueFormatter={valueFormatter}
      />
    </>
  );
}

export interface Preset {
  month: number;
  year: number;
}
