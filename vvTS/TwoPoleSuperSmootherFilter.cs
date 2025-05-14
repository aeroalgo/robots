using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200019E RID: 414
	[HandlerCategory("vvAverages"), HandlerName("TwoPoleSuperSmootherFilter")]
	public class TwoPoleSuperSmootherFilter : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000D1C RID: 3356 RVA: 0x00039A98 File Offset: 0x00037C98
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("twopolesmoother", new string[]
			{
				this.CutoffPeriod.ToString(),
				sec.get_CacheName()
			}, () => TwoPoleSuperSmootherFilter.Gen2PoleSSFilter(sec, this.CutoffPeriod));
		}

		// Token: 0x06000D1B RID: 3355 RVA: 0x00039924 File Offset: 0x00037B24
		public static IList<double> Gen2PoleSSFilter(ISecurity src, int cutoffperiod)
		{
			double[] array = new double[src.get_Bars().Count];
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			double num = Math.Atan(1.0);
			double num2 = 45.0 / num;
			double num3 = 1.0 / num2;
			double num4 = Math.Atan(1.0) * 4.0;
			double num5 = Math.Exp(-Math.Sqrt(2.0) * num4 / (double)cutoffperiod);
			double num6 = 2.0 * num5 * Math.Cos(num3 * Math.Sqrt(2.0) * 180.0 / (double)cutoffperiod);
			double num7 = num6;
			double num8 = -num5 * num5;
			double num9 = 1.0 - num7 - num8;
			for (int i = 0; i < src.get_Bars().Count; i++)
			{
				if (i < 4)
				{
					array[i] = (lowPrices[i] + highPrices[i]) / 2.0;
				}
				else
				{
					array[i] = num9 * ((lowPrices[i] + highPrices[i]) / 2.0) + num7 * array[i - 1] + num8 * array[i - 2];
				}
			}
			return array;
		}

		// Token: 0x17000445 RID: 1093
		public IContext Context
		{
			// Token: 0x06000D1D RID: 3357 RVA: 0x00039AFC File Offset: 0x00037CFC
			get;
			// Token: 0x06000D1E RID: 3358 RVA: 0x00039B04 File Offset: 0x00037D04
			set;
		}

		// Token: 0x17000444 RID: 1092
		[HandlerParameter(true, "15", Min = "1", Max = "70", Step = "1")]
		public int CutoffPeriod
		{
			// Token: 0x06000D19 RID: 3353 RVA: 0x00039911 File Offset: 0x00037B11
			get;
			// Token: 0x06000D1A RID: 3354 RVA: 0x00039919 File Offset: 0x00037B19
			set;
		}
	}
}
