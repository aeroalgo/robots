using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200019C RID: 412
	[HandlerCategory("vvAverages"), HandlerName("TwoPoleButterworthFilter")]
	public class TwoPoleButterworthFilter : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000D0E RID: 3342 RVA: 0x000395EC File Offset: 0x000377EC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("twopolebutter", new string[]
			{
				this.CutoffPeriod.ToString(),
				sec.get_CacheName()
			}, () => TwoPoleButterworthFilter.Gen2PoleButtFilter(sec, this.CutoffPeriod));
		}

		// Token: 0x06000D0D RID: 3341 RVA: 0x00039418 File Offset: 0x00037618
		public static IList<double> Gen2PoleButtFilter(ISecurity src, int cutoffperiod)
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
			double num9 = (1.0 - num6 + num5 * num5) / 4.0;
			for (int i = 0; i < src.get_Bars().Count; i++)
			{
				if (i < 4)
				{
					array[i] = (lowPrices[i] + highPrices[i]) / 2.0;
				}
				else
				{
					array[i] = num9 * ((lowPrices[i] + highPrices[i]) / 2.0 + 2.0 * ((lowPrices[i - 1] + highPrices[i - 1]) / 2.0) + (lowPrices[i - 2] + highPrices[i - 2]) / 2.0) + num7 * array[i - 1] + num8 * array[i - 2];
				}
			}
			return array;
		}

		// Token: 0x17000441 RID: 1089
		public IContext Context
		{
			// Token: 0x06000D0F RID: 3343 RVA: 0x00039650 File Offset: 0x00037850
			get;
			// Token: 0x06000D10 RID: 3344 RVA: 0x00039658 File Offset: 0x00037858
			set;
		}

		// Token: 0x17000440 RID: 1088
		[HandlerParameter(true, "15", Min = "1", Max = "70", Step = "1")]
		public int CutoffPeriod
		{
			// Token: 0x06000D0B RID: 3339 RVA: 0x00039405 File Offset: 0x00037605
			get;
			// Token: 0x06000D0C RID: 3340 RVA: 0x0003940D File Offset: 0x0003760D
			set;
		}
	}
}
