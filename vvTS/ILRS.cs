using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200017A RID: 378
	[HandlerCategory("vvAverages"), HandlerName("ILRS")]
	public class ILRS : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000BF1 RID: 3057 RVA: 0x00033452 File Offset: 0x00031652
		public IList<double> Execute(IList<double> src)
		{
			return ILRS.GenILRS(src, this.Period);
		}

		// Token: 0x06000BEF RID: 3055 RVA: 0x00033344 File Offset: 0x00031544
		public static IList<double> GenILRS(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = ILRS.iILRS(src, period, i);
				}
			}
			return array;
		}

		// Token: 0x06000BF0 RID: 3056 RVA: 0x0003338C File Offset: 0x0003158C
		public static double iILRS(IList<double> price, int period, int barNum)
		{
			double num = (double)(period * (period - 1)) * 0.5;
			double num2 = (double)((period - 1) * period * (2 * period - 1)) / 6.0;
			double num3 = 0.0;
			double num4 = 0.0;
			for (int i = 0; i < period; i++)
			{
				num3 += (double)i * price[barNum - i];
				num4 += price[barNum - i];
			}
			double num5 = (double)period * num3 - num * num4;
			double num6 = num * num - (double)period * num2;
			double num7;
			if (num6 != 0.0)
			{
				num7 = num5 / num6;
			}
			else
			{
				num7 = 0.0;
			}
			return num7 + SMA.iSMA(price, period, barNum);
		}

		// Token: 0x170003EB RID: 1003
		[HandlerParameter(true, "15", Min = "3", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000BED RID: 3053 RVA: 0x00033332 File Offset: 0x00031532
			get;
			// Token: 0x06000BEE RID: 3054 RVA: 0x0003333A File Offset: 0x0003153A
			set;
		}
	}
}
