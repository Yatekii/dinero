import "./Dashboard.css";
import { PortfolioSummaryResponse } from "../bindings/PortfolioSummaryResponse";
import { useLoaderData } from "react-router-dom";
import { SpendOverMonth } from "./SpendOverMonth";
import NetWorth from "./NetWorth";
import { Preset, SpendBreakdown } from "./SpendBreakdown";
import { useState } from "react";

// eslint-disable-next-line react-refresh/only-export-components
export async function dataLoader() {
  const response = await fetch("http://127.0.0.1:3000/data");
  const data = (await response.json()) as PortfolioSummaryResponse;

  return data;
}

function Dashboard() {
  const data = useLoaderData() as Awaited<ReturnType<typeof dataLoader>>;

  const [presets, setPresets] = useState([] as Preset[]);

  return (
    <>
      <div className="mt-5 w-full" />
      <NetWorth total_balance={data.total_balance} />
      <div className="mt-5 w-full" />
      <SpendOverMonth spend_per_month={data.spend_per_month} />
      <SpendBreakdown
        spend_per_month={data.spend_per_month}
        presets={presets}
      />
    </>
  );
}

export default Dashboard;
