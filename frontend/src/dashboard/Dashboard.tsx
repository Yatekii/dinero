import "./Dashboard.css";
import { PortfolioSummaryResponse } from "../bindings/PortfolioSummaryResponse";
import { useLoaderData } from "react-router-dom";
import { SpendOverMonth } from "./SpendOverMonth";
import NetWorth from "./NetWorth";
import { SpendBreakdown } from "./SpendBreakdown";
import { useState } from "react";
import { Tab, TabGroup, TabList, Title } from "@tremor/react";
import { SpendBreakdownTransactions } from "./SpendBreakdownTransactions";
import { API_URL } from "../main";
import { authenticatedFetch } from "../lib/auth";
import { withAuth } from "../lib/routerAuth";

// eslint-disable-next-line react-refresh/only-export-components
export const dataLoader = withAuth(async () => {
  const response = await authenticatedFetch(`${API_URL}/data`);
  
  if (!response.ok) {
    throw new Error(`Failed to fetch data: ${response.status} ${response.statusText}`);
  }
  
  const data = (await response.json()) as PortfolioSummaryResponse;
  return data;
});

function Dashboard() {
  const data = useLoaderData() as Awaited<ReturnType<typeof dataLoader>>;

  const currentMonth = new Date().getMonth() + 1;
  const currentYear = new Date().getFullYear();
  const months = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];

  const years = [] as number[];

  for (const ys of Object.values(data.spend_per_month.months)) {
    for (const y of Object.keys(ys)) {
      if (!years.includes(+y)) {
        years.push(+y);
      }
    }
  }
  years.sort();

  const [month, setMonth] = useState(currentMonth);
  const [year, setYear] = useState(currentYear);

  return (
    <>
      <div className="mt-5 w-full" />
      <NetWorth
        totalBalance={data.total_balance}
        totalPrediction={data.total_prediction}
        baseCurrency={data.base_currency}
      />

      <div className="mt-5 w-full" />
      <SpendOverMonth spendPerMonth={data.spend_per_month} />

      <div className="mt-5 w-full" />
      <Title>Spend Breakdown</Title>
      <TabGroup
        className="mt-5 mb-2"
        onIndexChange={(i) => setYear(years[i])}
        index={years.findIndex((y) => y == year)}
      >
        <TabList variant="solid" defaultValue="1">
          {years.map((year) => (
            <Tab value={year} key={year}>
              {year}
            </Tab>
          ))}
        </TabList>
      </TabGroup>
      <TabGroup onIndexChange={(i) => setMonth(i + 1)} index={month - 1}>
        <TabList variant="solid" defaultValue="1">
          {months.map((month, i) => (
            <Tab value={i} key={month}>
              {month}
            </Tab>
          ))}
        </TabList>
      </TabGroup>
      <SpendBreakdown
        spendPerMonth={data.spend_per_month}
        presets={[{ month, year }]}
      />
      <SpendBreakdownTransactions
        spendPerMonth={data.spend_per_month}
        presets={[{ month, year }]}
      />
    </>
  );
}

export default Dashboard;
