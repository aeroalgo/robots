using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000196 RID: 406
	[HandlerCategory("vvAverages"), HandlerName("SMA")]
	public class SMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CDD RID: 3293 RVA: 0x000389A4 File Offset: 0x00036BA4
		public IList<double> Execute(IList<double> price)
		{
			return this.Context.GetData("SMA", new string[]
			{
				this.Period.ToString(),
				price.GetHashCode().ToString()
			}, () => SMA.GenSMA(price, this.Period));
		}

		// Token: 0x06000CDC RID: 3292 RVA: 0x000388D0 File Offset: 0x00036AD0
		public static IList<double> GenSMA(IList<double> candles, int period)
		{
			int count = candles.Count;
			double[] array = new double[count];
			if (period < 4)
			{
				for (int i = 0; i < count; i++)
				{
					array[i] = SMA.iSMA_TSLab(candles, i, period);
				}
			}
			else
			{
				int num = Math.Min(count, period);
				double num2 = 0.0;
				for (int j = 0; j < num; j++)
				{
					num2 += candles[j];
					array[j] = num2 / (double)(j + 1);
				}
				for (int k = num; k < count; k++)
				{
					double num3 = candles[k];
					double num4 = candles[k - period];
					double num5 = array[k - 1];
					array[k] = num5 + (num3 - num4) / (double)period;
				}
			}
			return array;
		}

		// Token: 0x06000CDE RID: 3294 RVA: 0x00038A10 File Offset: 0x00036C10
		public static double iSMA(IList<double> src, int period, int barNum)
		{
			if (barNum < period)
			{
				period = barNum;
			}
			double num = 0.0;
			for (int i = 0; i < period; i++)
			{
				num += src[barNum - i];
			}
			return num / (double)period;
		}

		// Token: 0x06000CDF RID: 3295 RVA: 0x00038A4C File Offset: 0x00036C4C
		public static double iSMA_TSLab(IList<double> candles, int curbar, int period)
		{
			int num = curbar - period + 1;
			if (num < 0)
			{
				num = 0;
			}
			period = curbar - num + 1;
			double num2 = 0.0;
			while (num <= curbar && num < candles.Count)
			{
				num2 += candles[num++];
			}
			return num2 / (double)Math.Min(period, curbar + 1);
		}

		// Token: 0x17000434 RID: 1076
		public IContext Context
		{
			// Token: 0x06000CE0 RID: 3296 RVA: 0x00038AA1 File Offset: 0x00036CA1
			get;
			// Token: 0x06000CE1 RID: 3297 RVA: 0x00038AA9 File Offset: 0x00036CA9
			set;
		}

		// Token: 0x17000433 RID: 1075
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000CDA RID: 3290 RVA: 0x000388BD File Offset: 0x00036ABD
			get;
			// Token: 0x06000CDB RID: 3291 RVA: 0x000388C5 File Offset: 0x00036AC5
			set;
		}
	}
}
