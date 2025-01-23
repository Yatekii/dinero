export const valueFormatter = function (n: number) {
  return (
    "CHF " +
    new Intl.NumberFormat("de-CH", { maximumFractionDigits: 0 })
      .format(n)
      .toString()
  );
};
