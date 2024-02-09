using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200019D RID: 413
	[HandlerCategory("vvAverages"), HandlerName("ThreePoleButterworthFilter")]
	public class ThreePoleButterworthFilter : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000D15 RID: 3349 RVA: 0x00039894 File Offset: 0x00037A94
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("threepolebutter", new string[]
			{
				this.CutoffPeriod.ToString(),
				sec.get_CacheName()
			}, () => ThreePoleButterworthFilter.Gen3PoleButtFilter(sec, this.CutoffPeriod));
		}

		// Token: 0x06000D14 RID: 3348 RVA: 0x0003967C File Offset: 0x0003787C
		public static IList<double> Gen3PoleButtFilter(ISecurity src, int cutoffperiod)
		{
			double[] array = new double[src.get_Bars().Count];
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			double num = Math.Atan(1.0);
			double num2 = 45.0 / num;
			double num3 = 1.0 / num2;
			double num4 = Math.Atan(1.0) * 4.0;
			double num5 = Math.Exp(-num4 / (double)cutoffperiod);
			double num6 = 2.0 * num5 * Math.Cos(num3 * Math.Sqrt(3.0) * 180.0 / (double)cutoffperiod);
			double num7 = num5 * num5;
			double num8 = num6 + num7;
			double num9 = -(num7 + num6 * num7);
			double num10 = num7 * num7;
			double num11 = (1.0 - num6 + num7) * (1.0 - num7) / 8.0;
			for (int i = 0; i < src.get_Bars().Count; i++)
			{
				if (i < 4)
				{
					array[i] = (lowPrices[i] + highPrices[i]) / 2.0;
				}
				else
				{
					array[i] = num11 * ((lowPrices[i] + highPrices[i]) / 2.0 + 3.0 * ((lowPrices[i - 1] + highPrices[i - 1]) / 2.0) + 3.0 * ((lowPrices[i - 2] + highPrices[i - 2]) / 2.0) + (lowPrices[i - 3] + highPrices[i - 3]) / 2.0) + num8 * array[i - 1] + num9 * array[i - 2] + num10 * array[i - 3];
				}
			}
			return array;
		}

		// Token: 0x17000443 RID: 1091
		public IContext Context
		{
			// Token: 0x06000D16 RID: 3350 RVA: 0x000398F8 File Offset: 0x00037AF8
			get;
			// Token: 0x06000D17 RID: 3351 RVA: 0x00039900 File Offset: 0x00037B00
			set;
		}

		// Token: 0x17000442 RID: 1090
		[HandlerParameter(true, "15", Min = "1", Max = "70", Step = "1")]
		public int CutoffPeriod
		{
			// Token: 0x06000D12 RID: 3346 RVA: 0x00039669 File Offset: 0x00037869
			get;
			// Token: 0x06000D13 RID: 3347 RVA: 0x00039671 File Offset: 0x00037871
			set;
		}
	}
}
