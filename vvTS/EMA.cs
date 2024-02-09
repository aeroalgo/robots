using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200015D RID: 349
	[HandlerCategory("vvAverages"), HandlerName("EMA")]
	public class EMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B0B RID: 2827 RVA: 0x0002D6CC File Offset: 0x0002B8CC
		public static IList<double> EMA_MT(IList<double> src, double period)
		{
			double[] array = new double[src.Count];
			double num = 2.0 / (1.0 + period);
			array[0] = src[0];
			for (int i = 1; i < src.Count; i++)
			{
				array[i] = array[i - 1] + num * (src[i] - array[i - 1]);
			}
			return array;
		}

		// Token: 0x06000B0C RID: 2828 RVA: 0x0002D730 File Offset: 0x0002B930
		public static IList<double> EMA_TSLab(IList<double> candles, int period)
		{
			int count = candles.Count;
			double[] array = new double[count];
			int num = Math.Min(count, period);
			double num2 = 0.0;
			for (int i = 0; i < num; i++)
			{
				num2 += candles[i];
				array[i] = num2 / (double)(i + 1);
			}
			double num3 = 2.0 / (1.0 + (double)period);
			for (int j = num; j < count; j++)
			{
				double num4 = candles[j];
				double num5 = array[j - 1];
				array[j] = num3 * (num4 - num5) + num5;
			}
			return array;
		}

		// Token: 0x06000B0E RID: 2830 RVA: 0x0002D862 File Offset: 0x0002BA62
		public IList<double> Execute(IList<double> src)
		{
			return EMA.GenEMA(src, this.EMAperiod, this.Zerolag);
		}

		// Token: 0x06000B01 RID: 2817 RVA: 0x0002D3E5 File Offset: 0x0002B5E5
		public static IList<double> GenEMA(IList<double> src, double period)
		{
			return EMA.EMA_MT(src, period);
		}

		// Token: 0x06000B03 RID: 2819 RVA: 0x0002D40C File Offset: 0x0002B60C
		public static IList<double> GenEMA(IList<double> src, int period)
		{
			return EMA.EMA_MT(src, Convert.ToDouble(period));
		}

		// Token: 0x06000B00 RID: 2816 RVA: 0x0002D3D1 File Offset: 0x0002B5D1
		public static IList<double> GenEMA(IList<double> src, double period, bool zerolag)
		{
			if (zerolag)
			{
				return EMA.ZLEMA(src, period);
			}
			return EMA.EMA_MT(src, period);
		}

		// Token: 0x06000B02 RID: 2818 RVA: 0x0002D3EE File Offset: 0x0002B5EE
		public static IList<double> GenEMA(IList<double> src, int period, bool zerolag)
		{
			if (zerolag)
			{
				return EMA.ZLEMA(src, Convert.ToDouble(period));
			}
			return EMA.EMA_MT(src, Convert.ToDouble(period));
		}

		// Token: 0x06000B04 RID: 2820 RVA: 0x0002D41C File Offset: 0x0002B61C
		public static double iEMA(IList<double> price, IList<double> emabuf, int period, int bar)
		{
			double result = 0.0;
			if (bar <= 2)
			{
				result = price[bar];
			}
			else if (bar > 2)
			{
				result = emabuf[bar - 1] + 2.0 / (double)(1 + period) * (price[bar] - emabuf[bar - 1]);
			}
			return result;
		}

		// Token: 0x06000B05 RID: 2821 RVA: 0x0002D474 File Offset: 0x0002B674
		public static double iEMA(IList<double> price, IList<double> emabuf, double period, int bar)
		{
			double result = 0.0;
			if (bar <= 2)
			{
				result = price[bar];
			}
			else if (bar > 2)
			{
				result = emabuf[bar - 1] + 2.0 / (1.0 + period) * (price[bar] - emabuf[bar - 1]);
			}
			return result;
		}

		// Token: 0x06000B06 RID: 2822 RVA: 0x0002D4D0 File Offset: 0x0002B6D0
		public static double iEMA(double price, double prevema, int period, int bar)
		{
			double result = 0.0;
			if (bar <= 2)
			{
				result = price;
			}
			else if (bar > 2)
			{
				result = prevema + 2.0 / (double)(1 + period) * (price - prevema);
			}
			return result;
		}

		// Token: 0x06000B07 RID: 2823 RVA: 0x0002D50C File Offset: 0x0002B70C
		public static double iEMA(double price, double prevema, double period, int bar)
		{
			double result = 0.0;
			if (bar <= 2)
			{
				result = price;
			}
			else if (bar > 2)
			{
				result = prevema + 2.0 / (1.0 + period) * (price - prevema);
			}
			return result;
		}

		// Token: 0x06000B08 RID: 2824 RVA: 0x0002D54C File Offset: 0x0002B74C
		public static double iREMA(double price, IList<double> MaArray, int period, double lambda, int barNum)
		{
			double num = 2.0 / (double)(period + 1);
			double result = 0.0;
			if (barNum <= 3)
			{
				result = price;
			}
			else if (barNum > 3)
			{
				result = (MaArray[barNum - 1] * (1.0 + 2.0 * lambda) + num * (price - MaArray[barNum - 1]) - lambda * MaArray[barNum - 2]) / (1.0 + lambda);
			}
			return result;
		}

		// Token: 0x06000B09 RID: 2825 RVA: 0x0002D5CC File Offset: 0x0002B7CC
		public static double iZeroLagEMA(IList<double> price, IList<double> MaBuffer, int period, int barNum)
		{
			double num = 2.0 / (double)(1 + period);
			int num2 = Convert.ToInt32(0.5 * (double)(period - 1));
			double result = 0.0;
			if (barNum <= num2)
			{
				result = price[barNum];
			}
			else if (barNum > num2)
			{
				result = num * (2.0 * price[barNum] - price[barNum - num2]) + (1.0 - num) * MaBuffer[barNum - 1];
			}
			return result;
		}

		// Token: 0x06000B0A RID: 2826 RVA: 0x0002D650 File Offset: 0x0002B850
		public static double iZeroLagEMA(IList<double> price, double prevEma, int period, int barNum)
		{
			double num = 2.0 / (double)(1 + period);
			int num2 = Convert.ToInt32(0.5 * (double)(period - 1));
			double result = 0.0;
			if (barNum <= num2)
			{
				result = price[barNum];
			}
			else if (barNum > num2)
			{
				result = num * (2.0 * price[barNum] - price[barNum - num2]) + (1.0 - num) * prevEma;
			}
			return result;
		}

		// Token: 0x06000B0D RID: 2829 RVA: 0x0002D7D0 File Offset: 0x0002B9D0
		public static IList<double> ZLEMA(IList<double> src, double period)
		{
			double[] array = new double[src.Count];
			double num = 2.0 / (period + 1.0);
			int num2 = Convert.ToInt32((period - 1.0) / 2.0);
			for (int i = num2; i < src.Count; i++)
			{
				array[i] = array[i - 1] + num * (src[i] + (src[i] - src[i - num2]) - array[i - 1]);
			}
			return array;
		}

		// Token: 0x170003A6 RID: 934
		public IContext Context
		{
			// Token: 0x06000B0F RID: 2831 RVA: 0x0002D876 File Offset: 0x0002BA76
			get;
			// Token: 0x06000B10 RID: 2832 RVA: 0x0002D87E File Offset: 0x0002BA7E
			set;
		}

		// Token: 0x170003A4 RID: 932
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public double EMAperiod
		{
			// Token: 0x06000AFC RID: 2812 RVA: 0x0002D3AF File Offset: 0x0002B5AF
			get;
			// Token: 0x06000AFD RID: 2813 RVA: 0x0002D3B7 File Offset: 0x0002B5B7
			set;
		}

		// Token: 0x170003A5 RID: 933
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Zerolag
		{
			// Token: 0x06000AFE RID: 2814 RVA: 0x0002D3C0 File Offset: 0x0002B5C0
			get;
			// Token: 0x06000AFF RID: 2815 RVA: 0x0002D3C8 File Offset: 0x0002B5C8
			set;
		}
	}
}
