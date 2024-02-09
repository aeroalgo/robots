using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200006C RID: 108
	[HandlerCategory("vvIndicators"), HandlerName("VKWBands IBS")]
	public class VKWBandsIBS : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060003D1 RID: 977 RVA: 0x00014EF1 File Offset: 0x000130F1
		public IList<double> Execute(ISecurity src)
		{
			return VKWBandsIBS.GenVKWBandsIBS(src, this.Context, this.RangePeriod, this.SmoothPeriod, this.SmoothMode, this.Per, this.Output, this.Chart);
		}

		// Token: 0x060003D0 RID: 976 RVA: 0x00014D40 File Offset: 0x00012F40
		public static IList<double> GenVKWBandsIBS(ISecurity src, IContext ctx, int _RangePeriod, int _SmoothPeriod, int _SmoothMode, int _Per, int _Output, int _Chart)
		{
			int count = src.get_Bars().Count;
			IList<double> list = new double[count];
			IList<double> list2 = IBS.GenIBS(src, _Per, ctx);
			IList<double> candles = vvSeries.Highest(list2, _RangePeriod);
			IList<double> candles2 = vvSeries.Lowest(list2, _RangePeriod);
			IList<double> list3 = SMA.GenSMA(candles, _SmoothPeriod);
			IList<double> list4 = SMA.GenSMA(candles2, _SmoothPeriod);
			for (int i = 1; i < count; i++)
			{
				list[i] = 0.0;
				if (list2[i] < list3[i] && list2[i - 1] > list3[i - 1])
				{
					list[i] = -2.0;
				}
				if (list2[i] > list4[i] && list2[i - 1] < list4[i - 1])
				{
					list[i] = 2.0;
				}
			}
			if (_Chart > 0)
			{
				IPane pane = ctx.CreatePane("VKWIBS", 35.0, false, false);
				pane.AddList(string.Concat(new string[]
				{
					"VKWIBS(len:",
					_Per.ToString(),
					",rng:",
					_RangePeriod.ToString(),
					")"
				}), list2, 0, 14683430, 0, 0);
				pane.AddList("", list3, 0, 331942, 0, 0);
				pane.AddList("", list4, 0, 331942, 0, 0);
			}
			if (_Output == 1)
			{
				return list3;
			}
			if (_Output == 2)
			{
				return list4;
			}
			if (_Output == 3)
			{
				return list;
			}
			return list2;
		}

		// Token: 0x17000149 RID: 329
		[HandlerParameter(true, "0", NotOptimized = true)]
		public int Chart
		{
			// Token: 0x060003CE RID: 974 RVA: 0x00014D2F File Offset: 0x00012F2F
			get;
			// Token: 0x060003CF RID: 975 RVA: 0x00014D37 File Offset: 0x00012F37
			set;
		}

		// Token: 0x1700014A RID: 330
		public IContext Context
		{
			// Token: 0x060003D2 RID: 978 RVA: 0x00014F23 File Offset: 0x00013123
			get;
			// Token: 0x060003D3 RID: 979 RVA: 0x00014F2B File Offset: 0x0001312B
			set;
		}

		// Token: 0x17000148 RID: 328
		[HandlerParameter(true, "3", NotOptimized = true)]
		public int Output
		{
			// Token: 0x060003CC RID: 972 RVA: 0x00014D1E File Offset: 0x00012F1E
			get;
			// Token: 0x060003CD RID: 973 RVA: 0x00014D26 File Offset: 0x00012F26
			set;
		}

		// Token: 0x17000147 RID: 327
		[HandlerParameter(true, "5", Min = "0", Max = "10", Step = "1")]
		public int Per
		{
			// Token: 0x060003CA RID: 970 RVA: 0x00014D0D File Offset: 0x00012F0D
			get;
			// Token: 0x060003CB RID: 971 RVA: 0x00014D15 File Offset: 0x00012F15
			set;
		}

		// Token: 0x17000144 RID: 324
		[HandlerParameter(true, "25", Min = "5", Max = "50", Step = "1")]
		public int RangePeriod
		{
			// Token: 0x060003C4 RID: 964 RVA: 0x00014CDA File Offset: 0x00012EDA
			get;
			// Token: 0x060003C5 RID: 965 RVA: 0x00014CE2 File Offset: 0x00012EE2
			set;
		}

		// Token: 0x17000146 RID: 326
		[HandlerParameter(true, "0", Min = "0", Max = "4", Step = "1")]
		public int SmoothMode
		{
			// Token: 0x060003C8 RID: 968 RVA: 0x00014CFC File Offset: 0x00012EFC
			get;
			// Token: 0x060003C9 RID: 969 RVA: 0x00014D04 File Offset: 0x00012F04
			set;
		}

		// Token: 0x17000145 RID: 325
		[HandlerParameter(true, "3", Min = "0", Max = "10", Step = "1")]
		public int SmoothPeriod
		{
			// Token: 0x060003C6 RID: 966 RVA: 0x00014CEB File Offset: 0x00012EEB
			get;
			// Token: 0x060003C7 RID: 967 RVA: 0x00014CF3 File Offset: 0x00012EF3
			set;
		}
	}
}
