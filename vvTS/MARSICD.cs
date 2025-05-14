using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000034 RID: 52
	[HandlerCategory("vvIndicators"), HandlerName("MARSICD")]
	public class MARSICD : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001E8 RID: 488 RVA: 0x000092CC File Offset: 0x000074CC
		public IList<double> Execute(ISecurity src)
		{
			return MARSICD.GenMARSICD(src, this.Context, this.Alpha, this.ARsi_Period, this.RRsi_Period, this.AMa_Period, this.RMa_Period, this.sMaD_Period, this.Output, this.Chart);
		}

		// Token: 0x060001E7 RID: 487 RVA: 0x00008F8C File Offset: 0x0000718C
		public static IList<double> GenMARSICD(ISecurity sec, IContext ctx, double alpha, int _ARsi_Period, int _RRsi_Period, int _AMa_Period, int _RMa_Period, int _sMaD_Period, int _Output, int _Chart)
		{
			int count = sec.get_Bars().Count;
			IList<double> arg_2A_0 = sec.get_ClosePrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			IList<double> P = vvSeries.WeightedClose(sec);
			IList<double> data = ctx.GetData("rsi", new string[]
			{
				_ARsi_Period.ToString(),
				P.GetHashCode().ToString()
			}, () => RSI.GenRSI(P, _ARsi_Period, 0, 0, 100, false));
			IList<double> data2 = ctx.GetData("rsi", new string[]
			{
				_RRsi_Period.ToString(),
				P.GetHashCode().ToString()
			}, () => RSI.GenRSI(P, _RRsi_Period, 0, 0, 100, false));
			for (int i = 0; i < count; i++)
			{
				array[i] = 100.0 - data2[i];
			}
			IList<double> list = LWMA.GenWMA(data, _AMa_Period);
			IList<double> list2 = LWMA.GenWMA(array, _RMa_Period);
			for (int j = 0; j < count; j++)
			{
				array2[j] = list[j] - list2[j];
			}
			IList<double> result = LWMA.GenWMA(array2, _sMaD_Period);
			for (int k = 1; k < count; k++)
			{
				array3[k] = 0.0;
				if (list[k] > list2[k] && list[k - 1] < list2[k - 1])
				{
					array3[k] = 1.0;
				}
				if (list[k] < list2[k] && list[k - 1] > list2[k - 1])
				{
					array3[k] = -1.0;
				}
			}
			if (_Chart > 0)
			{
				IPane pane = ctx.CreatePane("", 30.0, false, false);
				IGraphList graphList = pane.AddList("Delta", array2, 1, 11121916, 0, 0);
				graphList.set_Opacity(50);
				pane.AddList("ARsi(" + _ARsi_Period.ToString() + ")", data, 0, 8618885, 0, 0);
				pane.AddList("RRsi(" + _RRsi_Period.ToString() + ")", array, 0, 8618885, 0, 0);
				IGraphList graphList2 = pane.AddList("AMa(" + _AMa_Period.ToString() + ")", list, 0, 240421, 0, 0);
				IGraphList graphList3 = pane.AddList("RMa(" + _RMa_Period.ToString() + ")", list2, 0, 14025743, 0, 0);
				graphList2.set_Thickness(2);
				graphList3.set_Thickness(2);
			}
			if (_Output == 0)
			{
				return data;
			}
			if (_Output == 1)
			{
				return list;
			}
			if (_Output == 2)
			{
				return array;
			}
			if (_Output == 3)
			{
				return list2;
			}
			if (_Output == 4)
			{
				return array2;
			}
			if (_Output == 5)
			{
				return result;
			}
			if (_Output == 6)
			{
				return array3;
			}
			return data;
		}

		// Token: 0x170000A3 RID: 163
		[HandlerParameter(true, "0.07", Min = "0", Max = "1", Step = "0.01")]
		public double Alpha
		{
			// Token: 0x060001E1 RID: 481 RVA: 0x00008F21 File Offset: 0x00007121
			get;
			// Token: 0x060001E2 RID: 482 RVA: 0x00008F29 File Offset: 0x00007129
			set;
		}

		// Token: 0x170000A0 RID: 160
		[HandlerParameter(true, "5", Min = "0", Max = "30", Step = "1")]
		public int AMa_Period
		{
			// Token: 0x060001DB RID: 475 RVA: 0x00008EEE File Offset: 0x000070EE
			get;
			// Token: 0x060001DC RID: 476 RVA: 0x00008EF6 File Offset: 0x000070F6
			set;
		}

		// Token: 0x1700009E RID: 158
		[HandlerParameter(true, "14", Min = "0", Max = "30", Step = "1")]
		public int ARsi_Period
		{
			// Token: 0x060001D7 RID: 471 RVA: 0x00008ECC File Offset: 0x000070CC
			get;
			// Token: 0x060001D8 RID: 472 RVA: 0x00008ED4 File Offset: 0x000070D4
			set;
		}

		// Token: 0x170000A5 RID: 165
		[HandlerParameter(true, "0", Min = "0", Max = "30", Step = "1")]
		public int Chart
		{
			// Token: 0x060001E5 RID: 485 RVA: 0x00008F43 File Offset: 0x00007143
			get;
			// Token: 0x060001E6 RID: 486 RVA: 0x00008F4B File Offset: 0x0000714B
			set;
		}

		// Token: 0x170000A6 RID: 166
		public IContext Context
		{
			// Token: 0x060001E9 RID: 489 RVA: 0x00009315 File Offset: 0x00007515
			get;
			// Token: 0x060001EA RID: 490 RVA: 0x0000931D File Offset: 0x0000751D
			set;
		}

		// Token: 0x170000A4 RID: 164
		[HandlerParameter(true, "0", Min = "0", Max = "30", Step = "1")]
		public int Output
		{
			// Token: 0x060001E3 RID: 483 RVA: 0x00008F32 File Offset: 0x00007132
			get;
			// Token: 0x060001E4 RID: 484 RVA: 0x00008F3A File Offset: 0x0000713A
			set;
		}

		// Token: 0x170000A1 RID: 161
		[HandlerParameter(true, "5", Min = "0", Max = "30", Step = "1")]
		public int RMa_Period
		{
			// Token: 0x060001DD RID: 477 RVA: 0x00008EFF File Offset: 0x000070FF
			get;
			// Token: 0x060001DE RID: 478 RVA: 0x00008F07 File Offset: 0x00007107
			set;
		}

		// Token: 0x1700009F RID: 159
		[HandlerParameter(true, "14", Min = "0", Max = "30", Step = "1")]
		public int RRsi_Period
		{
			// Token: 0x060001D9 RID: 473 RVA: 0x00008EDD File Offset: 0x000070DD
			get;
			// Token: 0x060001DA RID: 474 RVA: 0x00008EE5 File Offset: 0x000070E5
			set;
		}

		// Token: 0x170000A2 RID: 162
		[HandlerParameter(true, "5", Min = "0", Max = "30", Step = "1")]
		public int sMaD_Period
		{
			// Token: 0x060001DF RID: 479 RVA: 0x00008F10 File Offset: 0x00007110
			get;
			// Token: 0x060001E0 RID: 480 RVA: 0x00008F18 File Offset: 0x00007118
			set;
		}
	}
}
