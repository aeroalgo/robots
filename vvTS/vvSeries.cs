using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000095 RID: 149
	public static class vvSeries
	{
		// Token: 0x06000557 RID: 1367 RVA: 0x0001AF30 File Offset: 0x00019130
		public static IList<double> CalcMACD(IList<double> src, int p1, int p2)
		{
			IList<double> list = EMA.GenEMA(src, p1);
			IList<double> list2 = EMA.GenEMA(src, p2);
			double[] array = new double[list.Count];
			for (int i = 0; i < list.Count; i++)
			{
				array[i] = list[i] - list2[i];
			}
			return array;
		}

		// Token: 0x06000550 RID: 1360 RVA: 0x0001ABE0 File Offset: 0x00018DE0
		public static IList<double> CustomTypicalPrice(IList<Bar> candles)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (candles[i].get_High() + candles[i].get_Low() + candles[i].get_Close() + candles[i].get_Open()) / 4.0;
			}
			return array;
		}

		// Token: 0x06000551 RID: 1361 RVA: 0x0001AC48 File Offset: 0x00018E48
		public static IList<double> CustomTypicalPrice(ISecurity src)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (src.get_HighPrices()[i] + src.get_LowPrices()[i] + src.get_ClosePrices()[i] + src.get_OpenPrices()[i]) / 4.0;
			}
			return array;
		}

		// Token: 0x06000554 RID: 1364 RVA: 0x0001AD8C File Offset: 0x00018F8C
		public static IList<double> CustomWeightedClose(IList<Bar> candles)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (candles[i].get_Close() * 2.0 + candles[i].get_High() + candles[i].get_Low() + candles[i].get_Open()) / 5.0;
			}
			return array;
		}

		// Token: 0x06000555 RID: 1365 RVA: 0x0001AE00 File Offset: 0x00019000
		public static IList<double> CustomWeightedClose(ISecurity src)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (src.get_ClosePrices()[i] * 2.0 + src.get_HighPrices()[i] + src.get_LowPrices()[i] + src.get_LowPrices()[i]) / 5.0;
			}
			return array;
		}

		// Token: 0x0600055C RID: 1372 RVA: 0x0001B234 File Offset: 0x00019434
		public static IList<double> DrawTrendLine(IList<double> price, double Aprice, int Abarnum, double Bprice, int Bbarnum)
		{
			int count = price.Count;
			int num = Math.Max(Bbarnum - Abarnum, 1);
			double num2 = (Bprice - Aprice) / (double)num;
			double[] array = new double[count];
			for (int i = 0; i < num; i++)
			{
				if (i > 0)
				{
					array[i] = array[i - 1] + num2;
				}
				else
				{
					array[i] = Aprice;
				}
			}
			return array;
		}

		// Token: 0x06000556 RID: 1366 RVA: 0x0001AE78 File Offset: 0x00019078
		public static IList<double> GetPrice(ISecurity src, int PriceType = 0)
		{
			int arg_0B_0 = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> arg_19_0 = src.get_OpenPrices();
			IList<double> arg_20_0 = src.get_LowPrices();
			IList<double> arg_27_0 = src.get_HighPrices();
			if (PriceType < 1 || PriceType > 8)
			{
				return src.get_ClosePrices();
			}
			if (PriceType == 1)
			{
				return src.get_OpenPrices();
			}
			if (PriceType == 2)
			{
				return src.get_LowPrices();
			}
			if (PriceType == 3)
			{
				return src.get_HighPrices();
			}
			if (PriceType == 4)
			{
				return vvSeries.MedianPrice(src.get_Bars());
			}
			if (PriceType == 5)
			{
				return vvSeries.TypicalPrice(src.get_Bars());
			}
			if (PriceType == 6)
			{
				return vvSeries.WeightedClose(src.get_Bars());
			}
			if (PriceType == 7)
			{
				return vvSeries.CustomTypicalPrice(src.get_Bars());
			}
			if (PriceType == 8)
			{
				return vvSeries.CustomWeightedClose(src.get_Bars());
			}
			return closePrices;
		}

		// Token: 0x06000544 RID: 1348 RVA: 0x0001A878 File Offset: 0x00018A78
		public static IList<double> Highest(IList<double> candles, int period)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = vvSeries.iHighest(candles, i, period);
			}
			return array;
		}

		// Token: 0x06000546 RID: 1350 RVA: 0x0001A8EC File Offset: 0x00018AEC
		public static double HighestBarNum(IList<double> candles, int curbar, int period)
		{
			double num = -1.7976931348623157E+308;
			int num2 = 0;
			int num3 = curbar - period + 1;
			if (num3 < 0)
			{
				num3 = 0;
			}
			for (int i = curbar; i >= num3; i--)
			{
				if (num < candles[i])
				{
					num = candles[i];
					num2 = i;
				}
			}
			return (double)num2;
		}

		// Token: 0x06000547 RID: 1351 RVA: 0x0001A934 File Offset: 0x00018B34
		public static int HighestBarNumInt(IList<double> candles, int curbar, int period)
		{
			double num = -1.7976931348623157E+308;
			int result = 0;
			int num2 = curbar - period + 1;
			if (num2 < 0)
			{
				num2 = 0;
			}
			for (int i = curbar; i >= num2; i--)
			{
				if (num < candles[i])
				{
					num = candles[i];
					result = i;
				}
			}
			return result;
		}

		// Token: 0x06000545 RID: 1349 RVA: 0x0001A8AC File Offset: 0x00018AAC
		public static double iHighest(IList<double> candles, int curbar, int period)
		{
			double num = -1.7976931348623157E+308;
			int num2 = curbar - period + 1;
			if (num2 < 0)
			{
				num2 = 0;
			}
			for (int i = num2; i <= curbar; i++)
			{
				num = Math.Max(num, candles[i]);
			}
			return num;
		}

		// Token: 0x06000549 RID: 1353 RVA: 0x0001A9B0 File Offset: 0x00018BB0
		public static double iLowest(IList<double> candles, int curbar, int period)
		{
			double num = 1.7976931348623157E+308;
			int num2 = curbar - period + 1;
			if (num2 < 0)
			{
				num2 = 0;
			}
			for (int i = num2; i <= curbar; i++)
			{
				num = Math.Min(num, candles[i]);
			}
			return num;
		}

		// Token: 0x06000558 RID: 1368 RVA: 0x0001AF80 File Offset: 0x00019180
		public static double iMA(IList<double> P, IList<double> MaBuf, int MaType, int MaPeriod, int barNum, double MaParam1, double MaParam2)
		{
			double result;
			switch (MaType)
			{
			case 0:
				result = SMA.iSMA(P, MaPeriod, barNum);
				break;
			case 1:
				result = EMA.iEMA(P, MaBuf, MaPeriod, barNum);
				break;
			case 2:
				result = SMMA.iSMMA(P, MaBuf, MaPeriod, barNum);
				break;
			case 3:
				result = LWMA.iWMA(P, barNum, MaPeriod);
				break;
			case 4:
				result = LRMA.iLSMA(P, MaPeriod, barNum);
				break;
			case 5:
				result = EMA.iREMA(P[barNum], MaBuf, MaPeriod, MaParam1, barNum);
				break;
			case 6:
				result = SineWMA.iSineWMA(P, MaPeriod, barNum);
				break;
			case 7:
				result = EMA.iZeroLagEMA(P, MaBuf, MaPeriod, barNum);
				break;
			case 8:
				result = HullMA.iHMA(P, MaPeriod, barNum, MaParam1);
				break;
			default:
				result = SMA.iSMA(P, MaPeriod, barNum);
				break;
			}
			return result;
		}

		// Token: 0x0600055A RID: 1370 RVA: 0x0001B05C File Offset: 0x0001925C
		public static double iVsaSpread(ISecurity src, int barnum)
		{
			return src.get_HighPrices()[barnum] - src.get_LowPrices()[barnum];
		}

		// Token: 0x06000559 RID: 1369 RVA: 0x0001B04B File Offset: 0x0001924B
		public static double iVsaSpread(IList<double> highs, IList<double> lows, int barnum)
		{
			return highs[barnum] - lows[barnum];
		}

		// Token: 0x06000548 RID: 1352 RVA: 0x0001A97C File Offset: 0x00018B7C
		public static IList<double> Lowest(IList<double> candles, int period)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = Indicators.Lowest(candles, i, period);
			}
			return array;
		}

		// Token: 0x0600054A RID: 1354 RVA: 0x0001A9F0 File Offset: 0x00018BF0
		public static double LowestBarNum(IList<double> candles, int curbar, int period)
		{
			double num = 1.7976931348623157E+308;
			int num2 = 0;
			int num3 = curbar - period + 1;
			if (num3 < 0)
			{
				num3 = 0;
			}
			for (int i = curbar; i >= num3; i--)
			{
				if (num > candles[i])
				{
					num = candles[i];
					num2 = i;
				}
			}
			return (double)num2;
		}

		// Token: 0x0600054B RID: 1355 RVA: 0x0001AA38 File Offset: 0x00018C38
		public static int LowestBarNumInt(IList<double> candles, int curbar, int period)
		{
			double num = 1.7976931348623157E+308;
			int result = 0;
			int num2 = curbar - period + 1;
			if (num2 < 0)
			{
				num2 = 0;
			}
			for (int i = curbar; i >= num2; i--)
			{
				if (num > candles[i])
				{
					num = candles[i];
					result = i;
				}
			}
			return result;
		}

		// Token: 0x0600054C RID: 1356 RVA: 0x0001AA80 File Offset: 0x00018C80
		public static IList<double> MedianPrice(IList<Bar> candles)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (candles[i].get_High() + candles[i].get_Low()) / 2.0;
			}
			return array;
		}

		// Token: 0x0600054D RID: 1357 RVA: 0x0001AAD0 File Offset: 0x00018CD0
		public static IList<double> MedianPrice(ISecurity src)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (src.get_HighPrices()[i] + src.get_LowPrices()[i]) / 2.0;
			}
			return array;
		}

		// Token: 0x0600055B RID: 1371 RVA: 0x0001B078 File Offset: 0x00019278
		public static IList<double> ParabolicAdaptive(ISecurity src, double _AccStepFrom, double _AccStepTo, double _AccMax, IList<double> _Knorma, IContext ctx)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			bool flag = true;
			double num4 = _AccStepFrom;
			int i = 1;
			while (i < count)
			{
				double num5 = _AccStepFrom + _Knorma[i] * (_AccStepTo - _AccStepFrom);
				double num6 = lowPrices[i];
				double num7 = highPrices[i];
				double num8 = array[i - 1] + num4 * (num3 - array[i - 1]);
				if (flag)
				{
					if (num3 < num7 && num4 + num5 <= _AccMax)
					{
						num4 += num5;
					}
					if (num8 >= num6)
					{
						num4 = num5;
						flag = false;
						num3 = num6;
						num2 = num6;
						if (highPrices[i] < num)
						{
							array[i] = num;
						}
						else
						{
							array[i] = highPrices[i];
						}
					}
					else
					{
						if (num3 < num6 && num4 + num5 <= _AccMax)
						{
							num4 += num5;
						}
						if (num3 < num7)
						{
							num = num7;
							num3 = num7;
							goto IL_198;
						}
						goto IL_198;
					}
				}
				else
				{
					if (num3 > num6 && num4 + num5 <= _AccMax)
					{
						num4 += num5;
					}
					if (num8 <= num7)
					{
						num4 = num5;
						flag = true;
						num3 = num7;
						num = num7;
						if (lowPrices[i] > num2)
						{
							array[i] = num2;
						}
						else
						{
							array[i] = lowPrices[i];
						}
					}
					else
					{
						if (num3 > num7 && num4 + num5 <= _AccMax)
						{
							num4 += num5;
						}
						if (num3 > num6)
						{
							num2 = num6;
							num3 = num6;
							goto IL_198;
						}
						goto IL_198;
					}
				}
				IL_19E:
				i++;
				continue;
				IL_198:
				array[i] = num8;
				goto IL_19E;
			}
			return array;
		}

		// Token: 0x0600054E RID: 1358 RVA: 0x0001AB24 File Offset: 0x00018D24
		public static IList<double> TypicalPrice(IList<Bar> candles)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (candles[i].get_High() + candles[i].get_Low() + candles[i].get_Close()) / 3.0;
			}
			return array;
		}

		// Token: 0x0600054F RID: 1359 RVA: 0x0001AB80 File Offset: 0x00018D80
		public static IList<double> TypicalPrice(ISecurity src)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (src.get_HighPrices()[i] + src.get_LowPrices()[i] + src.get_ClosePrices()[i]) / 3.0;
			}
			return array;
		}

		// Token: 0x06000552 RID: 1362 RVA: 0x0001ACB8 File Offset: 0x00018EB8
		public static IList<double> WeightedClose(IList<Bar> candles)
		{
			int count = candles.Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (candles[i].get_Close() * 2.0 + candles[i].get_High() + candles[i].get_Low()) / 4.0;
			}
			return array;
		}

		// Token: 0x06000553 RID: 1363 RVA: 0x0001AD20 File Offset: 0x00018F20
		public static IList<double> WeightedClose(ISecurity src)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				array[i] = (src.get_ClosePrices()[i] * 2.0 + src.get_HighPrices()[i] + src.get_LowPrices()[i]) / 4.0;
			}
			return array;
		}
	}
}
