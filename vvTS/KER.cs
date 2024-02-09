using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000023 RID: 35
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("ER (Kaufman)")]
	public class KER : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000141 RID: 321 RVA: 0x00006095 File Offset: 0x00004295
		public IList<double> Execute(IList<double> src)
		{
			return KER.GenKER(src, this.ERperiod, this.postSmooth);
		}

		// Token: 0x0600013C RID: 316 RVA: 0x00005F54 File Offset: 0x00004154
		public static IList<double> GenKER(IList<double> src, int _ERperiod, int postsmooth)
		{
			int count = src.Count;
			double[] array = new double[count];
			if (_ERperiod <= 0)
			{
				_ERperiod = 10;
			}
			for (int i = _ERperiod; i < count; i++)
			{
				array[i] = KER.iKER(src, i, _ERperiod);
			}
			IList<double> result = array;
			if (postsmooth > 0)
			{
				result = JMA.GenJMA(array, postsmooth, 100);
			}
			return result;
		}

		// Token: 0x0600013D RID: 317 RVA: 0x00005FA0 File Offset: 0x000041A0
		public static double iKER(IList<double> src, int barNum, int _ERperiod)
		{
			if (_ERperiod <= 0)
			{
				_ERperiod = 10;
			}
			double num = KER.NetPriceMovement(src, barNum, _ERperiod);
			double num2 = KER.Volatility(src, barNum, _ERperiod);
			if (num == 0.0)
			{
				num = 1E-09;
			}
			if (num2 == 0.0)
			{
				num2 = 1E-09;
			}
			return num / num2;
		}

		// Token: 0x0600013E RID: 318 RVA: 0x00005FF8 File Offset: 0x000041F8
		public static double iKERwithPeriod(IList<double> src, int barNum, int ERperiod, int MinPeriod, int MaxPeriod)
		{
			double num = KER.iKER(src, barNum, ERperiod);
			return (double)MaxPeriod + num * (double)(MinPeriod - MaxPeriod);
		}

		// Token: 0x0600013F RID: 319 RVA: 0x0000601B File Offset: 0x0000421B
		private static double NetPriceMovement(IList<double> src, int initialbar, int erperiod)
		{
			if (initialbar < erperiod - 1)
			{
				return 0.0;
			}
			return Math.Abs(src[initialbar] - src[initialbar - erperiod]);
		}

		// Token: 0x06000140 RID: 320 RVA: 0x00006044 File Offset: 0x00004244
		private static double Volatility(IList<double> src, int initialbar, int erperiod)
		{
			if (initialbar < erperiod - 1)
			{
				return 0.0;
			}
			double num = 0.0;
			for (int i = 0; i < erperiod; i++)
			{
				num += Math.Abs(src[initialbar - i] - src[initialbar - 1 - i]);
			}
			return num;
		}

		// Token: 0x17000068 RID: 104
		[HandlerParameter(true, "10", Min = "6", Max = "60", Step = "1")]
		public int ERperiod
		{
			// Token: 0x06000138 RID: 312 RVA: 0x00005F2F File Offset: 0x0000412F
			get;
			// Token: 0x06000139 RID: 313 RVA: 0x00005F37 File Offset: 0x00004137
			set;
		}

		// Token: 0x17000069 RID: 105
		[HandlerParameter(true, "0", Min = "0", Max = "10", Step = "1")]
		public int postSmooth
		{
			// Token: 0x0600013A RID: 314 RVA: 0x00005F40 File Offset: 0x00004140
			get;
			// Token: 0x0600013B RID: 315 RVA: 0x00005F48 File Offset: 0x00004148
			set;
		}
	}
}
