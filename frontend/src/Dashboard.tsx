import "./Dashboard.css";
import { Card, Title } from "@tremor/react";
import { AreaChart } from "@tremor/react";
import { useState, useEffect } from "react";

function Dashboard() {
  const [data, setData] = useState("");

  useEffect(() => {
    // React advises to declare the async function directly inside useEffect
    async function fetchData() {
      const response = await fetch("http://127.0.0.1:3000/data");
      const data = await response.json();
      console.log(data);
      const chartdata = [];
      for (let i = 0; i < data.timestamps.length; i++) {
        chartdata.push({
          date: new Date(
            data.timestamps[i] * 24 * 60 * 60 * 1000
          ).toDateString(),
        });
        for (const entry of data.balances) {
          chartdata[i][entry.name] = entry.series[i] ?? 0;
        }
      }
      setData(chartdata);
    }

    fetchData();
  }, []);

  if (!data) {
    return "Loading ...";
  }

  const valueFormatter = function (number) {
    return "CHF " + new Intl.NumberFormat("us").format(number).toString();
  };

  console.log(data);

  return (
    <>
      <Card>
        <Title>Net worth over time (CHF)</Title>
        <AreaChart
          className="h-72 mt-4"
          data={data}
          index="date"
          yAxisWidth={65}
          categories={[
            "Neon",
            "UBS business [USD]",
            "UBS business [CHF]",
            "UBS private [CHF]",
          ]}
          connectNulls={true}
          colors={["indigo", "cyan", "orange", "yellow"]}
          valueFormatter={valueFormatter}
          stack="true"
        />
      </Card>
    </>
  );
}

export default Dashboard;
