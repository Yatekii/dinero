import { DonutChart, Title } from "@tremor/react";
import { SpendPerMonth } from "../bindings/SpendPerMonth";

export function SpendBreakdown({
  spend_per_month,
  presets,
}: {
  spend_per_month: SpendPerMonth;
  presets: Preset[];
}) {
  const data = [] as { name: string; value: number }[];

  if (presets.length === 0) {
    for (const y of Object.values(spend_per_month.months)) {
      console.log(y);
      for (const c of Object.values(y)) {
        for (const [cat, amount] of Object.entries(c)) {
          const elementIndex = data.findIndex((d) => d.name == cat);
          if (elementIndex != -1) {
            console.log(data[elementIndex].name, amount);
            data[elementIndex].value += amount;
          } else {
            console.log({ name: cat, value: amount });
            data.push({ name: cat, value: amount });
          }
        }
      }
    }
  } else {
    for (const preset of presets) {
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

  return (
    <>
      <Title>Spend Breakdown</Title>
      <DonutChart
        className="h-72 mt-4"
        data={data}
        index="name"
        valueFormatter={dataFormatter}
      />
    </>
  );
}

export interface Preset {
  month: number;
  year: number;
}

const dataFormatter = (number: number) =>
  `$ ${Intl.NumberFormat("de-DE").format(number).toString()}`;
