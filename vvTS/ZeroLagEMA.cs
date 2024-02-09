using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x020001AB RID: 427
	[HandlerCategory("vvAverages"), HandlerName("ZeroLag EMA (DEMAvar.)")]
	public class ZeroLagEMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D8C RID: 3468 RVA: 0x0003B493 File Offset: 0x00039693
		public IList<double> Execute(IList<double> src)
		{
			return ZeroLagEMA.GenZeroLagEMA(src, this.Period, this.SmoothPeriod, this.Context);
		}

		// Token: 0x06000D8B RID: 3467 RVA: 0x0003B3AC File Offset: 0x000395AC
		public static IList<double> GenZeroLagEMA(IList<double> src, int period1, int period2, IContext ctx)
		{
			double[] array = new double[src.Count];
			IList<double> EMA1 = ctx.GetData("ema1", new string[]
			{
				period1.ToString()
			}, () => Series.EMA(src, period1));
			IList<double> data = ctx.GetData("ema2", new string[]
			{
				period1.ToString(),
				period2.ToString()
			}, () => Series.EMA(EMA1, period2));
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = 2.0 * EMA1[i] - data[i];
			}
			return array;
		}

		// Token: 0x17000467 RID: 1127
		public IContext Context
		{
			// Token: 0x06000D8D RID: 3469 RVA: 0x0003B4AD File Offset: 0x000396AD
			get;
			// Token: 0x06000D8E RID: 3470 RVA: 0x0003B4B5 File Offset: 0x000396B5
			set;
		}

		// Token: 0x17000465 RID: 1125
		[HandlerParameter(true, "7", Min = "1", Max = "100", Step = "1")]
		public int Period
		{
			// Token: 0x06000D87 RID: 3463 RVA: 0x0003B359 File Offset: 0x00039559
			get;
			// Token: 0x06000D88 RID: 3464 RVA: 0x0003B361 File Offset: 0x00039561
			set;
		}

		// Token: 0x17000466 RID: 1126
		[HandlerParameter(true, "15", Min = "1", Max = "100", Step = "1")]
		public int SmoothPeriod
		{
			// Token: 0x06000D89 RID: 3465 RVA: 0x0003B36A File Offset: 0x0003956A
			get;
			// Token: 0x06000D8A RID: 3466 RVA: 0x0003B372 File Offset: 0x00039572
			set;
		}
	}
}
