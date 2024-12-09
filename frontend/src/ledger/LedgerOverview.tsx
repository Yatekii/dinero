import {
  DateRangePicker,
  DateRangePickerValue,
  DonutChart,
  Title,
} from "@tremor/react";
import { useLoaderData, useSearchParams } from "react-router-dom";
import { LedgerSummary } from "../bindings/LedgerSummary";
import { API_URL } from "../main";

// eslint-disable-next-line react-refresh/only-export-components
export async function ledgerOverviewLoader({
  request,
}: {
  request: { url: string };
}) {
  const searchParams = new URL(request.url).searchParams;
  const response = await fetch(
    `${API_URL}/ledgers/summary?${searchParams.toString()}`,
    {
      credentials: "include",
      redirect: "follow",
    }
  );
  const ledgers = (await response.json()) as LedgerSummary;

  return { ledgers };
}

export function LedgerOverview() {
  const { ledgers } = useLoaderData() as Awaited<
    ReturnType<typeof ledgerOverviewLoader>
  >;

  const [searchParams, setSearchParams] = useSearchParams();

  const selectDate = (v: DateRangePickerValue) => {
    const params = new URLSearchParams();
    if (v.from) {
      params.append("from", v.from.getTime().toString());
    }
    if (v.to) {
      params.append("to", v.to.getTime().toString());
    }
    setSearchParams(params);
  };

  const from = searchParams.get("from");
  const to = searchParams.get("to");

  const date = {
    from: from ? new Date(+from) : undefined,
    to: to ? new Date(+to) : undefined,
  };

  const expenses = [];
  const income = [];

  for (const [, ledger] of Object.entries(ledgers.spending)) {
    for (const [category, total] of Object.entries(ledger.categories)) {
      if (total > 0) {
        income.push({ name: category, value: total });
      } else {
        expenses.push({ name: category, value: -total });
      }
    }
  }

  return (
    <>
      <div className="space-x-3 max-w-md flex mb-3">
        <div>
          <p className="font-mono text-sm text-slate-500">from </p>
          <DateRangePicker
            value={date}
            onValueChange={selectDate}
            enableSelect={false}
          />
        </div>
      </div>

      <Title>Ledger Overview</Title>
      <DonutChart
        data={expenses}
        variant="pie"
        valueFormatter={dataFormatter}
        onValueChange={(v) => console.log(v)}
      />
    </>
  );
}

const dataFormatter = (number: number) =>
  `$ ${Intl.NumberFormat("de-DE").format(number).toString()}`;
