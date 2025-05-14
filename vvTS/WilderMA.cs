using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A7 RID: 423
	[HandlerCategory("vvAverages"), HandlerName("Wilder's MA")]
	public class WilderMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000D6A RID: 3434 RVA: 0x0003AF5B File Offset: 0x0003915B
		public IList<double> Execute(IList<double> src)
		{
			return WilderMA.GenWilderMA(src, this.Period);
		}

		// Token: 0x06000D69 RID: 3433 RVA: 0x0003AF20 File Offset: 0x00039120
		public static IList<double> GenWilderMA(IList<double> src, int period)
		{
			IList<double> list = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				list[i] = WilderMA.iWilder(src, list, period, i);
			}
			return list;
		}

		// Token: 0x06000D6B RID: 3435 RVA: 0x0003AF6C File Offset: 0x0003916C
		public static double iWilder(IList<double> price, IList<double> wilderbuf, int period, int bar)
		{
			if (bar <= 1)
			{
				return price[bar];
			}
			double result;
			if (bar <= period)
			{
				result = SMA.iSMA(price, period, bar);
			}
			else
			{
				result = wilderbuf[bar - 1] + (price[bar] - wilderbuf[bar - 1]) / (double)period;
			}
			return result;
		}

		// Token: 0x1700045B RID: 1115
		[HandlerParameter(true, "10", Min = "2", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000D67 RID: 3431 RVA: 0x0003AF0D File Offset: 0x0003910D
			get;
			// Token: 0x06000D68 RID: 3432 RVA: 0x0003AF15 File Offset: 0x00039115
			set;
		}
	}
}
