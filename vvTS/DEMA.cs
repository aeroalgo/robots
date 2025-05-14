using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000161 RID: 353
	[HandlerCategory("vvAverages"), HandlerName("DEMA")]
	public class DEMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B2C RID: 2860 RVA: 0x0002DE74 File Offset: 0x0002C074
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("DEMA", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => DEMA.GenDEMA(src, this.Context, this.Period));
		}

		// Token: 0x06000B2B RID: 2859 RVA: 0x0002DD2C File Offset: 0x0002BF2C
		public static IList<double> GenDEMA(IList<double> src, IContext ctx, int period)
		{
			IList<double> ema1 = ctx.GetData("ema", new string[]
			{
				period.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.EMA(src, period));
			IList<double> data = ctx.GetData("ema", new string[]
			{
				period.ToString(),
				ema1.GetHashCode().ToString()
			}, () => Series.EMA(ema1, period));
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i <= period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = 2.0 * ema1[i] - data[i];
				}
			}
			return array;
		}

		// Token: 0x170003AF RID: 943
		public IContext Context
		{
			// Token: 0x06000B2D RID: 2861 RVA: 0x0002DEE0 File Offset: 0x0002C0E0
			get;
			// Token: 0x06000B2E RID: 2862 RVA: 0x0002DEE8 File Offset: 0x0002C0E8
			set;
		}

		// Token: 0x170003AE RID: 942
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000B29 RID: 2857 RVA: 0x0002DCEB File Offset: 0x0002BEEB
			get;
			// Token: 0x06000B2A RID: 2858 RVA: 0x0002DCF3 File Offset: 0x0002BEF3
			set;
		}
	}
}
