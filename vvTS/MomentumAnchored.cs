using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000039 RID: 57
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("Anchored Momentum")]
	public class MomentumAnchored : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600020E RID: 526 RVA: 0x00009B68 File Offset: 0x00007D68
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("momanchored", new string[]
			{
				this.MomPeriod.ToString(),
				this.Smooth.ToString(),
				src.GetHashCode().ToString()
			}, () => MomentumAnchored.GenMomentumAnchored(src, this.MomPeriod, this.Smooth, this.Context));
		}

		// Token: 0x0600020D RID: 525 RVA: 0x000099F4 File Offset: 0x00007BF4
		public static IList<double> GenMomentumAnchored(IList<double> src, int _momperiod, int _smooth, IContext ctx)
		{
			double[] array = new double[src.Count];
			int num = 2 * _momperiod + 1;
			IList<double> data = ctx.GetData("sma", new string[]
			{
				_momperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.SMA(src, _momperiod));
			IList<double> data2 = ctx.GetData("ema", new string[]
			{
				_smooth.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.EMA(src, _smooth));
			for (int i = 0; i < src.Count; i++)
			{
				if (i <= num)
				{
					array[i] = 0.0;
				}
				else
				{
					array[i] = 100.0 * (data2[i] / data[i] - 1.0);
				}
			}
			return array;
		}

		// Token: 0x170000B2 RID: 178
		public IContext Context
		{
			// Token: 0x0600020F RID: 527 RVA: 0x00009BE6 File Offset: 0x00007DE6
			get;
			// Token: 0x06000210 RID: 528 RVA: 0x00009BEE File Offset: 0x00007DEE
			set;
		}

		// Token: 0x170000B0 RID: 176
		[HandlerParameter(true, "11", Min = "8", Max = "20", Step = "1")]
		public int MomPeriod
		{
			// Token: 0x06000209 RID: 521 RVA: 0x000099A1 File Offset: 0x00007BA1
			get;
			// Token: 0x0600020A RID: 522 RVA: 0x000099A9 File Offset: 0x00007BA9
			set;
		}

		// Token: 0x170000B1 RID: 177
		[HandlerParameter(true, "6", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x0600020B RID: 523 RVA: 0x000099B2 File Offset: 0x00007BB2
			get;
			// Token: 0x0600020C RID: 524 RVA: 0x000099BA File Offset: 0x00007BBA
			set;
		}
	}
}
