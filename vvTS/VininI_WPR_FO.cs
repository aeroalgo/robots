using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000141 RID: 321
	[HandlerCategory("vvIndicators"), HandlerName("VininI_WPR_FO")]
	public class VininI_WPR_FO : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060009E7 RID: 2535 RVA: 0x00028FDE File Offset: 0x000271DE
		public IList<double> Execute(ISecurity src)
		{
			return VininI_WPR_FO.GenWPR_FO(src, this.Context, this.WPR_Period, this.MA_Period);
		}

		// Token: 0x060009E6 RID: 2534 RVA: 0x00028EA0 File Offset: 0x000270A0
		public static IList<double> GenWPR_FO(ISecurity _sec, IContext ctx, int _WPR_Period, int _MA_Period)
		{
			int count = _sec.get_Bars().Count;
			Math.Max(_WPR_Period, _MA_Period);
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> data = ctx.GetData("wpr", new string[]
			{
				_WPR_Period.ToString(),
				_sec.get_CacheName()
			}, () => WPR.GenWPR(_sec, _WPR_Period, 0, 0, ctx));
			for (int i = 0; i < count; i++)
			{
				list2[i] = 0.1 * (data[i] + 50.0);
			}
			IList<double> list3 = EMA.GenEMA(list2, _MA_Period);
			for (int j = 2; j < count; j++)
			{
				list[j] = (Math.Exp(2.0 * list3[j]) - 1.0) / (Math.Exp(2.0 * list3[j]) + 1.0);
			}
			return list;
		}

		// Token: 0x1700033F RID: 831
		public IContext Context
		{
			// Token: 0x060009E8 RID: 2536 RVA: 0x00028FF8 File Offset: 0x000271F8
			get;
			// Token: 0x060009E9 RID: 2537 RVA: 0x00029000 File Offset: 0x00027200
			set;
		}

		// Token: 0x1700033E RID: 830
		[HandlerParameter(true, "9", Min = "3", Max = "50", Step = "1")]
		public int MA_Period
		{
			// Token: 0x060009E4 RID: 2532 RVA: 0x00028E69 File Offset: 0x00027069
			get;
			// Token: 0x060009E5 RID: 2533 RVA: 0x00028E71 File Offset: 0x00027071
			set;
		}

		// Token: 0x1700033D RID: 829
		[HandlerParameter(true, "5", Min = "3", Max = "25", Step = "1")]
		public int WPR_Period
		{
			// Token: 0x060009E2 RID: 2530 RVA: 0x00028E58 File Offset: 0x00027058
			get;
			// Token: 0x060009E3 RID: 2531 RVA: 0x00028E60 File Offset: 0x00027060
			set;
		}
	}
}
