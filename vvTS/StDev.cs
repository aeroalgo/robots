using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200005A RID: 90
	[HandlerCategory("vvIndicators"), HandlerName("StDev")]
	public class StDev : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600033B RID: 827 RVA: 0x00012A98 File Offset: 0x00010C98
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("StDev", new string[]
			{
				this.Period.ToString(),
				this.MaMethod.ToString(),
				src.GetHashCode().ToString()
			}, () => StDev.GenStDev(src, this.Period, this.MaMethod, this.Context));
		}

		// Token: 0x06000337 RID: 823 RVA: 0x000126DC File Offset: 0x000108DC
		public static IList<double> GenStDev(IList<double> src, int _StDevPeriod, int _MaMethod, IContext ctx)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> list = null;
			if (_MaMethod < 0 || _MaMethod > 3)
			{
				_MaMethod = 0;
			}
			switch (_MaMethod)
			{
			case 0:
				list = ctx.GetData("sma", new string[]
				{
					_StDevPeriod.ToString(),
					src.GetHashCode().ToString()
				}, () => SMA.GenSMA(src, _StDevPeriod));
				break;
			case 1:
				list = ctx.GetData("ema", new string[]
				{
					_StDevPeriod.ToString(),
					src.GetHashCode().ToString()
				}, () => EMA.GenEMA(src, _StDevPeriod, false));
				break;
			case 2:
				list = ctx.GetData("smma", new string[]
				{
					_StDevPeriod.ToString(),
					src.GetHashCode().ToString()
				}, () => SMMA.GenSMMA(src, _StDevPeriod, _StDevPeriod / 2));
				break;
			case 3:
				list = ctx.GetData("lwma", new string[]
				{
					_StDevPeriod.ToString(),
					src.GetHashCode().ToString()
				}, () => LWMA.GenWMA(src, _StDevPeriod));
				break;
			}
			for (int i = _StDevPeriod; i < count; i++)
			{
				double num = 0.0;
				for (int j = 0; j < _StDevPeriod; j++)
				{
					num += (src[i - j] - list[i]) * (src[i - j] - list[i]);
				}
				array[i] = Math.Sqrt(num / (double)_StDevPeriod);
			}
			return array;
		}

		// Token: 0x06000338 RID: 824 RVA: 0x00012934 File Offset: 0x00010B34
		public static IList<double> GenStDevOnMA(IList<double> src, IList<double> ma, int _StDevPeriod)
		{
			int count = src.Count;
			double[] array = new double[count];
			for (int i = _StDevPeriod; i < count; i++)
			{
				double num = 0.0;
				for (int j = 0; j < _StDevPeriod; j++)
				{
					num += (src[i - j] - ma[i]) * (src[i - j] - ma[i]);
				}
				array[i] = Math.Sqrt(num / (double)_StDevPeriod);
			}
			return array;
		}

		// Token: 0x06000339 RID: 825 RVA: 0x000129B4 File Offset: 0x00010BB4
		public static IList<double> GenStDev_TSLab(IList<double> candles, int period)
		{
			int count = candles.Count;
			double[] array = new double[count];
			IList<double> sMAs = SMA.GenSMA(candles, period);
			for (int i = 0; i < count; i++)
			{
				array[i] = StDev.iStDev(candles, sMAs, i, period);
			}
			return array;
		}

		// Token: 0x0600033A RID: 826 RVA: 0x000129F0 File Offset: 0x00010BF0
		public static double iStDev(IList<double> candles, IList<double> SMAs, int curbar, int period)
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
				double num3 = candles[num] - SMAs[curbar];
				num2 += num3 * num3;
				num++;
			}
			int num4 = Math.Min(period, curbar + 1);
			return Math.Sqrt(num2 / (double)num4 - 1.0);
		}

		// Token: 0x17000116 RID: 278
		public IContext Context
		{
			// Token: 0x0600033C RID: 828 RVA: 0x00012B16 File Offset: 0x00010D16
			get;
			// Token: 0x0600033D RID: 829 RVA: 0x00012B1E File Offset: 0x00010D1E
			set;
		}

		// Token: 0x17000115 RID: 277
		[HandlerParameter(true, "0", Min = "0", Max = "3", Step = "1", Name = "Mode:\n0-SMA,1-EMA\n2-SMMA,3-LWMA")]
		public int MaMethod
		{
			// Token: 0x06000335 RID: 821 RVA: 0x0001266E File Offset: 0x0001086E
			get;
			// Token: 0x06000336 RID: 822 RVA: 0x00012676 File Offset: 0x00010876
			set;
		}

		// Token: 0x17000114 RID: 276
		[HandlerParameter(true, "10", Min = "1", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000333 RID: 819 RVA: 0x0001265D File Offset: 0x0001085D
			get;
			// Token: 0x06000334 RID: 820 RVA: 0x00012665 File Offset: 0x00010865
			set;
		}
	}
}
