import { AreaChart, Title } from "@tremor/react";
import { PortfolioLedgersData } from "../bindings/PortfolioLedgersData";

interface ChartData {
  date: string;
  [x: string]: number | string;
}

export default function NetWorth({
  total_balance,
}: {
  total_balance: PortfolioLedgersData;
}) {
  const data = [] as ChartData[];

  for (let i = 0; i < total_balance.timestamps.length; i++) {
    data.push({
      date: new Date((total_balance.timestamps[i] ?? 0) * 1000).toDateString(),
    });
    for (const entry of total_balance.balances) {
      data[i][entry.name] = entry.series[i] ?? 0;
    }
  }

  const categories = [];
  for (const entry of total_balance.balances) {
    categories.push(entry.name);
  }

  return (
    <>
      <Title>Net worth over time (CHF)</Title>
      <AreaChart
        className="h-72 mt-4"
        data={data}
        index="date"
        yAxisWidth={65}
        categories={categories}
        connectNulls={true}
        colors={["indigo", "cyan", "orange", "yellow", "red"]}
        valueFormatter={valueFormatter}
        stack={true}
        xAxisLabel="Date"
        yAxisLabel="Net Worth"
      />
    </>
  );
}

const valueFormatter = function (n: number) {
  return "CHF " + new Intl.NumberFormat("de-DE").format(n).toString();
};
