using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000008 RID: 8
	[HandlerCategory("vvIndicators"), HandlerName("ATR"), InputInfo(0, "Инструмент")]
	public class ATR : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600003A RID: 58 RVA: 0x00002D54 File Offset: 0x00000F54

		// Token: 0x0600003C RID: 60 RVA: 0x00002E2C File Offset: 0x0000102C
		public IList<double> Execute(ISecurity sec)
		{
			if (!this.WATR)
			{
				return this.Context.GetData("atr", new string[]
				{
					this.Period.ToString(),
					this.Smooth.ToString(),
					sec.get_CacheName()
				}, () => ATR.GenATR(sec, this.Period, this.Smooth));
			}
			return this.Context.GetData("atr", new string[]
			{
				this.Period.ToString(),
				this.Smooth.ToString(),
				sec.get_CacheName()
			}, () => ATR.GenWATR(sec, this.Period, this.Smooth, this.Context));
		}

		// Token: 0x06000037 RID: 55 RVA: 0x00002C74 File Offset: 0x00000E74
		public static IList<double> GenATR(IList<Bar> candles, int period)
		{
			return ATR.ATR_TSLab(candles, period);
		}

		// Token: 0x06000036 RID: 54 RVA: 0x00002C4C File Offset: 0x00000E4C
		public static IList<double> GenATR(ISecurity sec, int period, int smooth)
		{
			IList<double> list = ATR.ATR_TSLab(sec.get_Bars(), period);
			if (smooth > 0)
			{
				list = JMA.GenJMA(list, smooth, 0);
			}
			return list;
		}

		// Token: 0x06000038 RID: 56 RVA: 0x00002C80 File Offset: 0x00000E80
		public static IList<double> GenWATR(ISecurity sec, int period, int smooth, IContext ctx)
		{
			if (sec.get_ClosePrices().Count < period || sec.get_ClosePrices().Count < 2)
			{
				return null;
			}
			IList<double> list = LWMA.GenWMA(TrueRange.GenTrueRange(sec.get_Bars()), period);
			if (smooth > 0)
			{
				list = JMA.GenJMA(list, smooth, 0);
			}
			return list;
		}

		// Token: 0x06000039 RID: 57 RVA: 0x00002CCC File Offset: 0x00000ECC
		public static double iATR(ISecurity sec, int period, int barNum)
		{
			if (barNum < period)
			{
				period = barNum;
			}
			List<double> list = new List<double>(period);
			for (int i = barNum - period + 1; i <= barNum; i++)
			{
				double val = sec.get_HighPrices()[i];
				double val2 = sec.get_LowPrices()[i];
				double val3 = sec.get_ClosePrices()[i - 1];
				list.Add(Math.Max(val, val3) - Math.Min(val2, val3));
			}
			IList<double> list2 = SMA.GenSMA(list, period);
			return list2[list2.Count - 1];
		}
		public static IList<double> ATR_TSLab(IList<Bar> candles, int period)
		{
			double prevAtr = 0.0;
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 1; i < count; i++)
			{
				prevAtr = (array[i] = ATR.iATR(candles, i, period, prevAtr));
			}
			return array;
		}
		// Token: 0x0600003B RID: 59 RVA: 0x00002D98 File Offset: 0x00000F98
		public static double iATR(IList<Bar> candles, int start, int period, double prevAtr)
		{
			if (start - period > 0)
			{
				return (prevAtr * (double)(period - 1) + TrueRange.iTrueRange(candles, start)) / (double)period;
			}
			double num = prevAtr * (double)start;
			num += TrueRange.iTrueRange(candles, start);
			return num / (double)(start + 1);
		}

		// Token: 0x17000012 RID: 18
		public IContext Context
		{
			// Token: 0x0600003D RID: 61 RVA: 0x00002F05 File Offset: 0x00001105
			get;
			// Token: 0x0600003E RID: 62 RVA: 0x00002F0D File Offset: 0x0000110D
			set;
		}

		// Token: 0x1700000F RID: 15
		[HandlerParameter(true, "10", Min = "5", Max = "100", Step = "5")]
		public int Period
		{
			// Token: 0x06000030 RID: 48 RVA: 0x00002C18 File Offset: 0x00000E18
			get;
			// Token: 0x06000031 RID: 49 RVA: 0x00002C20 File Offset: 0x00000E20
			set;
		}

		// Token: 0x17000010 RID: 16
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000032 RID: 50 RVA: 0x00002C29 File Offset: 0x00000E29
			get;
			// Token: 0x06000033 RID: 51 RVA: 0x00002C31 File Offset: 0x00000E31
			set;
		}

		// Token: 0x17000011 RID: 17
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool WATR
		{
			// Token: 0x06000034 RID: 52 RVA: 0x00002C3A File Offset: 0x00000E3A
			get;
			// Token: 0x06000035 RID: 53 RVA: 0x00002C42 File Offset: 0x00000E42
			set;
		}
	}
}
