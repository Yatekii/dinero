import { AreaChart, Title } from "@tremor/react";
import { PortfolioLedgersData } from "../bindings/PortfolioLedgersData";

interface ChartData {
  date: string;
  [x: string]: number | string;
}

export default function NetWorth({
  totalBalance,
}: {
  totalBalance: PortfolioLedgersData;
}) {
  const data = [] as ChartData[];

  for (let i = 0; i < totalBalance.timestamps.length; i++) {
    data.push({
      date: new Date((totalBalance.timestamps[i] ?? 0) * 1000).toDateString(),
    });
    for (const entry of totalBalance.balances) {
      data[i][entry.name] = entry.series[i] ?? 0;
    }
  }

  const categories = [];
  for (const entry of totalBalance.balances) {
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
