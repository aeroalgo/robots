using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000140 RID: 320
	[HandlerCategory("vvIndicators"), HandlerName("VininI Impuls")]
	public class VininImpulse : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060009DE RID: 2526 RVA: 0x00028E13 File Offset: 0x00027013
		public IList<double> Execute(ISecurity src)
		{
			return VininImpulse.GenVImpulse(src, this.Context, this.Bar0, this.Bar1, this.Bar2, this.MaPeriod, this.Output);
		}

		// Token: 0x060009DD RID: 2525 RVA: 0x00028D30 File Offset: 0x00026F30
		public static IList<double> GenVImpulse(ISecurity sec, IContext ctx, int bar0, int bar1, int bar2, int maperiod, int output)
		{
			int count = sec.get_Bars().Count;
			IList<double> closePrices = sec.get_ClosePrices();
			IList<double> openPrices = sec.get_OpenPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			int num = Math.Max(bar0, Math.Max(bar1, bar2));
			for (int i = num; i < count; i++)
			{
				array[i] = (closePrices[i] - openPrices[i - bar0]) / (double)bar0;
				array2[i] = (closePrices[i] - openPrices[i - bar1]) / (double)bar1;
				array3[i] = (closePrices[i] - openPrices[i - bar2]) / (double)bar2;
			}
			IList<double> result = SMA.GenSMA(array, maperiod);
			IList<double> result2 = SMA.GenSMA(array2, maperiod);
			IList<double> result3 = SMA.GenSMA(array3, maperiod);
			if (output == 1)
			{
				return result2;
			}
			if (output == 2)
			{
				return result3;
			}
			return result;
		}

		// Token: 0x17000337 RID: 823
		[HandlerParameter(true, "30", Min = "30", Max = "60", Step = "10")]
		public int Bar0
		{
			// Token: 0x060009D3 RID: 2515 RVA: 0x00028CD8 File Offset: 0x00026ED8
			get;
			// Token: 0x060009D4 RID: 2516 RVA: 0x00028CE0 File Offset: 0x00026EE0
			set;
		}

		// Token: 0x17000338 RID: 824
		[HandlerParameter(true, "90", Min = "30", Max = "90", Step = "10")]
		public int Bar1
		{
			// Token: 0x060009D5 RID: 2517 RVA: 0x00028CE9 File Offset: 0x00026EE9
			get;
			// Token: 0x060009D6 RID: 2518 RVA: 0x00028CF1 File Offset: 0x00026EF1
			set;
		}

		// Token: 0x17000339 RID: 825
		[HandlerParameter(true, "180", Min = "60", Max = "180", Step = "10")]
		public int Bar2
		{
			// Token: 0x060009D7 RID: 2519 RVA: 0x00028CFA File Offset: 0x00026EFA
			get;
			// Token: 0x060009D8 RID: 2520 RVA: 0x00028D02 File Offset: 0x00026F02
			set;
		}

		// Token: 0x1700033C RID: 828
		public IContext Context
		{
			// Token: 0x060009DF RID: 2527 RVA: 0x00028E3F File Offset: 0x0002703F
			get;
			// Token: 0x060009E0 RID: 2528 RVA: 0x00028E47 File Offset: 0x00027047
			set;
		}

		// Token: 0x1700033A RID: 826
		[HandlerParameter(true, "5", Min = "1", Max = "30", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x060009D9 RID: 2521 RVA: 0x00028D0B File Offset: 0x00026F0B
			get;
			// Token: 0x060009DA RID: 2522 RVA: 0x00028D13 File Offset: 0x00026F13
			set;
		}

		// Token: 0x1700033B RID: 827
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1")]
		public int Output
		{
			// Token: 0x060009DB RID: 2523 RVA: 0x00028D1C File Offset: 0x00026F1C
			get;
			// Token: 0x060009DC RID: 2524 RVA: 0x00028D24 File Offset: 0x00026F24
			set;
		}
	}
}
