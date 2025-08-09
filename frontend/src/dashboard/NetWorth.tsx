import { useState, useMemo, useCallback } from "react";
import { Title } from "@tremor/react";
import { PortfolioLedgersData } from "../bindings/PortfolioLedgersData";
import { AreaChart, TooltipProps, AreaChartEventProps } from "../components/AreaChart";
import { cx } from "../lib/utils";
import { getColorClassName, AvailableChartColorsKeys } from "../lib/chartUtils";
import { PortfolioLedgerData } from "../bindings/PortfolioLedgerData";
import { Currency } from "../bindings/Currency";
import { valueFormatter } from "../lib/numbers";

interface ChartData {
  date: Date;
  [x: string]: number | string | Date;
}

export default function NetWorth({
  totalBalance,
  totalPrediction,
  baseCurrency,
}: {
  totalBalance: PortfolioLedgersData;
  totalPrediction: PortfolioLedgerData;
  baseCurrency: Currency;
}) {
  // State to track which accounts are visible
  const [hiddenAccounts, setHiddenAccounts] = useState<Set<string>>(new Set());
  
  // Memoize data processing to reduce lag
  const { data, categories } = useMemo(() => {
    const processedData = [] as ChartData[];

    // Build data for ALL accounts, but set hidden ones to 0
    for (let i = 0; i < totalBalance.timestamps.length; i++) {
      processedData.push({
        date: new Date((totalBalance.timestamps[i] ?? 0) * 1000),
      });
      for (const entry of totalBalance.balances) {
        // Set value to 0 for hidden accounts, otherwise use actual value
        processedData[i][entry.name] = hiddenAccounts.has(entry.name) ? 0 : (entry.series[i] ?? 0);
      }
    }

    const lastDate = totalBalance.timestamps[totalBalance.timestamps.length - 1];
    const isPredictionVisible = !hiddenAccounts.has(totalPrediction.name);
    
    // Always add prediction data, but set to 0 if hidden
    for (let i = 0; i < totalPrediction.series.length; i++) {
      const date = new Date(lastDate * 1000);
      date.setDate(date.getDate() + i);
      processedData.push({
        date,
      });
      processedData[processedData.length - 1][totalPrediction.name] =
        isPredictionVisible ? (totalPrediction.series[i] ?? 0) : 0;
    }

    // Build categories from ALL accounts (keep labels visible)
    const processedCategories = [];
    for (const entry of totalBalance.balances) {
      processedCategories.push({ name: entry.name });
    }
    processedCategories.push({ name: totalPrediction.name, stack: "prediction" });

    return { data: processedData, categories: processedCategories };
  }, [totalBalance, totalPrediction, hiddenAccounts]);

  // Memoize current balance calculation
  const currentBalance = useMemo(() => {
    return totalBalance.balances
      .filter((entry) => !hiddenAccounts.has(entry.name))
      .map((b) => b.series[b.series.length - 1])
      .reduce((t, v) => t + v, 0)
      .toFixed(0);
  }, [totalBalance.balances, hiddenAccounts]);

  // Click handler for toggling account visibility
  const handleLegendClick = useCallback((categoryName: string) => {
    setHiddenAccounts((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(categoryName)) {
        newSet.delete(categoryName);
      } else {
        newSet.add(categoryName);
      }
      return newSet;
    });
  }, []);

  return (
    <>
      <div className="flex justify-between">
        <Title>Net worth over time (CHF)</Title>
        <Title>
          Current:{" "}
          {currentBalance}{" "}
          {baseCurrency}
        </Title>
      </div>
      <div className="relative">
        {/* Custom Legend */}
        <div className="flex flex-wrap justify-end gap-2 mb-4">
          {categories.map((category, index) => {
            const isHidden = hiddenAccounts.has(category.name);
            const colors: AvailableChartColorsKeys[] = ["blue", "emerald", "violet", "amber", "gray", "cyan", "pink", "lime", "fuchsia"];
            const color = colors[index % colors.length];
            
            return (
              <button
                key={category.name}
                onClick={() => handleLegendClick(category.name)}
                className={cx(
                  "group inline-flex flex-nowrap items-center gap-1.5 whitespace-nowrap rounded px-2 py-1 transition cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800",
                  isHidden && "opacity-50"
                )}
              >
                <span
                  className={cx(
                    "h-[3px] w-3.5 shrink-0 rounded-full",
                    getColorClassName(color, "bg")
                  )}
                  aria-hidden={true}
                />
                <span
                  className={cx(
                    "truncate whitespace-nowrap text-xs text-gray-700 dark:text-gray-300 group-hover:text-gray-900 dark:group-hover:text-gray-50"
                  )}
                >
                  {category.name}
                </span>
              </button>
            );
          })}
        </div>
        
        <AreaChart
          className="h-72"
          data={data}
          index="date"
          yAxisWidth={65}
          categories={categories}
          connectNulls={true}
          colors={[
            "blue",
            "emerald",
            "violet",
            "amber",
            "gray",
            "cyan",
            "pink",
            "lime",
            "fuchsia",
          ]}
          valueFormatter={valueFormatter}
          tickFormatter={tickFormatter}
          type="stacked"
          xAxisLabel="Date"
          yAxisLabel="Net Worth"
          customTooltip={Tooltip}
          minValue={0}
          tickGap={50}
          showLegend={false}
        />
      </div>
    </>
  );
}

const tickFormatter = function (n: unknown): string {
  const date = n as Date;
  return `${date.toLocaleString("default", {
    month: "short",
  })} ${date.getFullYear()}`;
};

const Tooltip = ({ payload, active, label }: TooltipProps) => {
  if (!active || !payload || payload.length === 0) return null;

  const total = payload
    .filter((p) => !p.category.includes("Prediction"))
    .map((p) => p.value)
    .reduce((t, v) => t + v, 0);

  return (
    <div
      className={cx(
        // base
        "rounded-md border text-sm shadow-md",
        // border color
        "border-gray-200 dark:border-gray-800",
        // background color
        "bg-white dark:bg-gray-950"
      )}
    >
      <div className={cx("border-b border-inherit px-4 py-2")}>
        <p
          className={cx(
            // base
            "font-medium",
            // text color
            "text-gray-900 dark:text-gray-50"
          )}
        >
          {`${label.getDate()} ${label.toLocaleString("default", {
            month: "short",
          })} ${label.getFullYear()}`}
        </p>
      </div>
      <div className={cx("border-b border-inherit px-4 py-2")}>
        <div className="flex items-center justify-between space-x-8">
          <div className="flex items-center space-x-2">
            <p
              className={cx(
                // base
                "whitespace-nowrap text-right",
                // text color
                "text-gray-700 dark:text-gray-300"
              )}
            >
              Total
            </p>
          </div>
          <p
            className={cx(
              // base
              "whitespace-nowrap text-right font-medium tabular-nums",
              // text color
              "text-gray-900 dark:text-gray-50"
            )}
          >
            {valueFormatter(total)}
          </p>
        </div>
      </div>
      <div className={cx("space-y-1 px-4 py-2")}>
        {payload.map(({ value, category, color }, index) => (
          <div
            key={`id-${index}`}
            className="flex items-center justify-between space-x-8"
          >
            <div className="flex items-center space-x-2">
              <span
                aria-hidden="true"
                className={cx(
                  "h-[3px] w-3.5 shrink-0 rounded-full",
                  getColorClassName(color, "bg")
                )}
              />
              <p
                className={cx(
                  // base
                  "whitespace-nowrap text-right",
                  // text color
                  "text-gray-700 dark:text-gray-300"
                )}
              >
                {category}
              </p>
            </div>
            <p
              className={cx(
                // base
                "whitespace-nowrap text-right font-medium tabular-nums",
                // text color
                "text-gray-900 dark:text-gray-50"
              )}
            >
              {valueFormatter(value)}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
};
