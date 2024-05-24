import "./Dashboard.css";
import { Title } from "@tremor/react";
import { AreaChart } from "@tremor/react";
import { PortfolioSummaryResponse } from "./bindings/PortfolioSummaryResponse";
import { useLoaderData } from "react-router-dom";

// eslint-disable-next-line react-refresh/only-export-components
export async function dataLoader() {
  const response = await fetch("http://127.0.0.1:3000/data");
  const data = ((await response.json()) as PortfolioSummaryResponse)
    .total_balance;
  const chartdata = [] as ChartData[];

  for (let i = 0; i < data.timestamps.length; i++) {
    chartdata.push({
      date: new Date(
        (data.timestamps[i] ?? 0) * 24 * 60 * 60 * 1000
      ).toDateString(),
    });
    for (const entry of data.balances) {
      chartdata[i][entry.name] = entry.series[i] ?? 0;
    }
  }

  const categories = [];
  for (const entry of data.balances) {
    categories.push(entry.name);
  }

  return { data: chartdata, categories };
}

function Dashboard() {
  const data = useLoaderData() as Awaited<ReturnType<typeof dataLoader>>;

  return (
    <>
      <Overview data={data.data} categories={data.categories} />
    </>
  );
}

interface ChartData {
  date: string;
  [x: string]: number | string;
}

function Overview({
  data,
  categories,
}: {
  data: ChartData[];
  categories: string[];
}) {
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
        colors={["indigo", "cyan", "orange", "yellow"]}
        valueFormatter={valueFormatter}
        stack={true}
      />
    </>
  );
}

const valueFormatter = function (n: number) {
  return "CHF " + new Intl.NumberFormat("us").format(n).toString();
};

export default Dashboard;
