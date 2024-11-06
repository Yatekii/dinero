import { Title } from "@tremor/react";
import { PortfolioLedgersData } from "../bindings/PortfolioLedgersData";
import { AreaChart, TooltipProps } from "../components/AreaChart";
import { cx } from "../lib/utils";
import { getColorClassName } from "../lib/chartUtils";

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
        type="stacked"
        xAxisLabel="Date"
        yAxisLabel="Net Worth"
        customTooltip={Tooltip}
      />
    </>
  );
}

const valueFormatter = function (n: number) {
  return "CHF " + new Intl.NumberFormat("de-DE").format(n).toString();
};

const Tooltip = ({ payload, active, label }: TooltipProps) => {
  if (!active || !payload || payload.length === 0) return null;

  const total = payload.map((p) => p.value).reduce((t, v) => t + v);

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
          {label}
        </p>
      </div>
      <div className={cx("border-b border-inherit px-4 py-2")}>
        <p
          className={cx(
            // base
            "font-medium",
            // text color
            "text-gray-900 dark:text-gray-50"
          )}
        >
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
        </p>
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
