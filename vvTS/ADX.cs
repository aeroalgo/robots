using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000004 RID: 4
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("ADX"), InputInfo(0, "Инструмент")]
	public class ADX : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000013 RID: 19 RVA: 0x0000299B File Offset: 0x00000B9B
		public IList<double> Execute(ISecurity src)
		{
			return this.GenAdvADX(src, this.Context, this.ADXperiod, this.Output, this.ADXRinterval);
		}

		// Token: 0x0600000E RID: 14 RVA: 0x0000211F File Offset: 0x0000031F
		public IList<double> GenAdvADX(ISecurity src, IContext ctx, int _ADXperiod, int _Output, int _ADXRinterval = 2)
		{
			return ADX.GenADX(src, _ADXperiod, _Output, _ADXRinterval);
		}

		// Token: 0x0600000F RID: 15 RVA: 0x0000212C File Offset: 0x0000032C
		public static IList<double> GenADX(ISecurity src, int period, int Mode, int _ADXRinterval = 2)
		{
			int count = src.get_Bars().Count;
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = new double[count];
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> list4 = EMA.GenEMA(TrueRange.GenTrueRange(src.get_Bars()), period);
			for (int i = 1; i < count; i++)
			{
				double num = highPrices[i] - highPrices[i - 1];
				double num2 = lowPrices[i - 1] - lowPrices[i];
				if ((num < 0.0 && num2 < 0.0) || num == num2)
				{
					num2 = (num = 0.0);
				}
				if (num2 > num)
				{
					num = 0.0;
				}
				if (num > num2)
				{
					num2 = 0.0;
				}
				list2[i] = num2;
				list[i] = num;
			}
			list = EMA.GenEMA(list, period);
			list2 = EMA.GenEMA(list2, period);
			for (int j = 0; j < count; j++)
			{
				list[j] = ((list4[j] == 0.0) ? 0.0 : (list[j] / list4[j]));
				list2[j] = ((list4[j] == 0.0) ? 0.0 : (list2[j] / list4[j]));
				list3[j] = ((list[j] == 0.0 && list2[j] == 0.0) ? 0.0 : (Math.Abs(list[j] - list2[j]) / (list[j] + list2[j]) * 100.0));
			}
			list3 = EMA.GenEMA(list3, period);
			if (Mode == 1)
			{
				return list;
			}
			if (Mode == 2)
			{
				return list2;
			}
			if (Mode == 3)
			{
				IList<double> list5 = new double[count];
				for (int k = 0; k < count; k++)
				{
					if (k < _ADXRinterval)
					{
						list5[k] = list3[k];
					}
					else
					{
						list5[k] = (list3[k] + list3[k - _ADXRinterval]) / 2.0;
					}
				}
				return list5;
			}
			return list3;
		}

		// Token: 0x06000012 RID: 18 RVA: 0x00002620 File Offset: 0x00000820
		public static IList<double> GenADXm(ISecurity src, int _period, int _Level, int _Chart, int _Output)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			IList<double> arg_19_0 = src.get_OpenPrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = new double[count];
			IList<double> list4 = new double[count];
			IList<double> list5 = new double[count];
			int num = 2;
			double num2 = 2.0 / (double)(_period + 1);
			list[num - 1] = 0.0;
			list2[num - 1] = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			double num5 = 0.0;
			int num6 = 0;
			int num7 = 0;
			for (int i = num; i < src.get_Bars().Count; i++)
			{
				double num8 = num3;
				double num9 = num4;
				double num10 = num5;
				double num11 = highPrices[i];
				double num12 = lowPrices[i];
				double num13 = num11 - highPrices[i - 1];
				double num14 = lowPrices[i - 1] - num12;
				if (num13 < 0.0)
				{
					num13 = 0.0;
				}
				if (num14 < 0.0)
				{
					num14 = 0.0;
				}
				if (num13 == num14)
				{
					num13 = 0.0;
					num14 = 0.0;
				}
				else if (num13 < num14)
				{
					num13 = 0.0;
				}
				else if (num14 < num13)
				{
					num14 = 0.0;
				}
				double val = Math.Abs(num11 - num12);
				double val2 = Math.Abs(num11 - closePrices[i - 1]);
				double val3 = Math.Abs(num12 - closePrices[i - 1]);
				double num15 = Math.Max(Math.Max(val, val2), val3);
				double num16;
				double num17;
				if (num15 == 0.0)
				{
					num16 = 0.0;
					num17 = 0.0;
				}
				else
				{
					num16 = 100.0 * num13 / num15;
					num17 = 100.0 * num14 / num15;
				}
				num3 = num8 + (num16 - num8) * num2;
				num4 = num9 + (num17 - num9) * num2;
				double value = num3 - num4;
				list3[i] = value;
				double num18 = Math.Abs(num3 + num4);
				double num19;
				if (num18 == 0.0)
				{
					num19 = 0.0;
				}
				else
				{
					num19 = 100.0 * (Math.Abs(value) / num18);
				}
				if (num3 > num4)
				{
					num6 = 1;
				}
				if (num3 < num4)
				{
					num6 = 2;
				}
				if (num6 == 2)
				{
					num19 = -num19;
				}
				num5 = num10 + (num19 - num10) * num2;
				list[i] = num5;
				if (list[i] > list[i - 1])
				{
					num7 = 1;
				}
				if (list[i] < list[i - 1])
				{
					if (num7 == 1)
					{
						list2[i - 1] = list[i - 1];
					}
					num7 = 2;
				}
				if (num7 == 2)
				{
					list2[i] = list[i];
				}
				else
				{
					list2[i] = 0.0;
				}
				list4[i] = (double)_Level;
				list5[i] = (double)(-(double)_Level);
			}
			if (_Output == 1)
			{
				return list;
			}
			if (_Output == 2)
			{
				return list2;
			}
			return list3;
		}

		// Token: 0x06000011 RID: 17 RVA: 0x000024A0 File Offset: 0x000006A0
		public static IList<double> GenADXxBB(ISecurity sec, IContext ctx, int _ADXperiod, int _Bandsperiod, int _Output)
		{
			IList<double> list = new double[sec.get_Bars().Count];
			IList<double> list2 = new double[sec.get_Bars().Count];
			IList<double> list3 = new double[sec.get_Bars().Count];
			IList<double> list4 = new double[sec.get_Bars().Count];
			IList<double> list5 = ADX.GenADX(sec, _ADXperiod, 0, 2);
			IList<double> list6 = ADX.GenADX(sec, _ADXperiod, 1, 2);
			IList<double> list7 = ADX.GenADX(sec, _ADXperiod, 2, 2);
			IList<double> list8 = BBands.GenBBands(sec.get_ClosePrices(), ctx, _Bandsperiod, 2.0, 1, 0);
			IList<double> list9 = BBands.GenBBands(sec.get_ClosePrices(), ctx, _Bandsperiod, 2.0, 2, 0);
			for (int i = 0; i < sec.get_Bars().Count; i++)
			{
				list4[i] = list8[i] - list9[i];
				if (list6[i] >= list7[i])
				{
					list[i] = list5[i] * list4[i];
					list2[i] = 0.0;
					list3[i] = list[i];
				}
				else
				{
					list[i] = 0.0;
					list2[i] = list5[i] * list4[i];
					list3[i] = -list2[i];
				}
			}
			if (_Output == 1)
			{
				return list;
			}
			if (_Output == 2)
			{
				return list2;
			}
			return list3;
		}

		// Token: 0x06000010 RID: 16 RVA: 0x0000239C File Offset: 0x0000059C
		public static IList<double> GenADX_WA(ISecurity src, IContext ctx, int _ADXperiod, int _Output)
		{
			IList<double> list = new double[src.get_Bars().Count];
			IList<double> list2 = new double[src.get_Bars().Count];
			IList<double> list3 = new double[src.get_Bars().Count];
			IList<double> list4 = ADX.GenADX(src, _ADXperiod, 0, 2);
			IList<double> list5 = ADX.GenADX(src, _ADXperiod, 1, 2);
			IList<double> list6 = ADX.GenADX(src, _ADXperiod, 2, 2);
			for (int i = 0; i < src.get_Bars().Count; i++)
			{
				if (list5[i] >= list6[i])
				{
					list[i] = list4[i];
					list2[i] = 0.0;
					list3[i] = list4[i];
				}
				else
				{
					list[i] = 0.0;
					list2[i] = list4[i];
					list3[i] = -list4[i];
				}
			}
			if (_Output == 1)
			{
				return list;
			}
			if (_Output == 2)
			{
				return list2;
			}
			return list3;
		}

		// Token: 0x17000002 RID: 2
		[HandlerParameter(true, "13", Min = "0", Max = "20", Step = "1")]
		public int ADXperiod
		{
			// Token: 0x06000008 RID: 8 RVA: 0x000020EC File Offset: 0x000002EC
			get;
			// Token: 0x06000009 RID: 9 RVA: 0x000020F4 File Offset: 0x000002F4
			set;
		}

		// Token: 0x17000003 RID: 3
		[HandlerParameter(true, "2", Min = "2", Max = "9", Step = "1")]
		public int ADXRinterval
		{
			// Token: 0x0600000A RID: 10 RVA: 0x000020FD File Offset: 0x000002FD
			get;
			// Token: 0x0600000B RID: 11 RVA: 0x00002105 File Offset: 0x00000305
			set;
		}

		// Token: 0x17000005 RID: 5
		public IContext Context
		{
			// Token: 0x06000014 RID: 20 RVA: 0x000029BC File Offset: 0x00000BBC
			get;
			// Token: 0x06000015 RID: 21 RVA: 0x000029C4 File Offset: 0x00000BC4
			set;
		}

		// Token: 0x17000004 RID: 4
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1", Name = "Output:\n0 ADX\n1 +DI\n2 -DI\n3 ADXR")]
		public int Output
		{
			// Token: 0x0600000C RID: 12 RVA: 0x0000210E File Offset: 0x0000030E
			get;
			// Token: 0x0600000D RID: 13 RVA: 0x00002116 File Offset: 0x00000316
			set;
		}
	}
}
