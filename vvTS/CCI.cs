using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200001B RID: 27
	[HandlerCategory("vvIndicators"), HandlerName("CCI")]
	public class CCI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x060000E7 RID: 231 RVA: 0x00004E64 File Offset: 0x00003064
		public IList<double> Execute(ISecurity sec)
		{
			return CCI.GenCCI(sec.get_Bars(), this.Period, this.postSmooth, this.postSmoothPhase);
		}

		// Token: 0x060000E3 RID: 227 RVA: 0x00004C50 File Offset: 0x00002E50
		public static IList<double> GenCCI(IList<Bar> candles, int _period, int _postsmooth, int _postsmoothphase = 100)
		{
			IList<double> list = TypicalPrice.GenTypicalPrice(candles);
			IList<double> list2 = SMA.GenSMA(list, _period);
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = list[i] - list2[i];
			}
			for (int j = 0; j < count; j++)
			{
				double num = CCI.MA(list, j, _period, list2[j]) * 0.015;
				array[j] = ((num == 0.0) ? 0.0 : (array[j] / num));
			}
			IList<double> result = array;
			if (_postsmooth > 0)
			{
				result = JMA.GenJMA(array, _postsmooth, _postsmoothphase);
			}
			return result;
		}

		// Token: 0x060000E4 RID: 228 RVA: 0x00004D04 File Offset: 0x00002F04
		public static double iCCI(ISecurity src, int period, int shift)
		{
			if (shift < period)
			{
				period = shift;
			}
			double result = 0.0;
			for (int i = 0; i < period; i++)
			{
				double num = 0.0;
				for (int j = 0; j < period; j++)
				{
					num += CCI.TypPr(src.get_Bars(), shift - j);
				}
				num /= (double)period;
				double num2 = 0.0;
				for (int k = 0; k < period; k++)
				{
					num2 += Math.Abs(CCI.TypPr(src.get_Bars(), shift - k) - num);
				}
				num2 /= (double)period;
				if (num2 != 0.0)
				{
					result = (CCI.TypPr(src.get_Bars(), shift) - num) / (0.015 * num2);
				}
				else
				{
					result = 0.0;
				}
			}
			return result;
		}

		// Token: 0x060000E6 RID: 230 RVA: 0x00004E08 File Offset: 0x00003008
		private static double MA(IList<double> candles, int curbar, int period, double sma)
		{
			int num = curbar - period + 1;
			if (num < 0)
			{
				num = 0;
			}
			period = curbar - num + 1;
			double num2 = 0.0;
			int count = candles.Count;
			while (num <= curbar && num < count)
			{
				num2 += Math.Abs(candles[num++] - sma);
			}
			return num2 / (double)Math.Max(1, period);
		}

		// Token: 0x060000E5 RID: 229 RVA: 0x00004DD3 File Offset: 0x00002FD3
		private static double TypPr(IList<Bar> candles, int curbar)
		{
			return (candles[curbar].get_Close() + candles[curbar].get_High() + candles[curbar].get_Low()) / 3.0;
		}

		// Token: 0x17000048 RID: 72
		[HandlerParameter(true, "14", Min = "3", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x060000DD RID: 221 RVA: 0x00004C1C File Offset: 0x00002E1C
			get;
			// Token: 0x060000DE RID: 222 RVA: 0x00004C24 File Offset: 0x00002E24
			set;
		}

		// Token: 0x17000049 RID: 73
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060000DF RID: 223 RVA: 0x00004C2D File Offset: 0x00002E2D
			get;
			// Token: 0x060000E0 RID: 224 RVA: 0x00004C35 File Offset: 0x00002E35
			set;
		}

		// Token: 0x1700004A RID: 74
		[HandlerParameter(true, "100", Min = "-100", Max = "100", Step = "25")]
		public int postSmoothPhase
		{
			// Token: 0x060000E1 RID: 225 RVA: 0x00004C3E File Offset: 0x00002E3E
			get;
			// Token: 0x060000E2 RID: 226 RVA: 0x00004C46 File Offset: 0x00002E46
			set;
		}
	}
}
