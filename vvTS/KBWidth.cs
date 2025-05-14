using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000122 RID: 290
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("KBWidth")]
	public class KBWidth : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000857 RID: 2135 RVA: 0x000235BA File Offset: 0x000217BA
		public IList<double> Execute(ISecurity src)
		{
			return KBWidth.GenKBWidth(src, this.Context, this.BandsPeriod, this.AtrPeriod, this.BandsMultiplier, this.NormPeriod, this.Smooth);
		}

		// Token: 0x06000856 RID: 2134 RVA: 0x00023490 File Offset: 0x00021690
		public static IList<double> GenKBWidth(ISecurity src, IContext ctx, int _Period, int _AtrPeriod, double _BandsMultiplier, int _NormPeriod, int _Smooth)
		{
			IList<double> list = KeltnerBands.GenKeltnerBands(src, ctx, _AtrPeriod, _Period, 2, _BandsMultiplier, 0);
			int count = list.Count;
			IList<double> list2 = KeltnerBands.GenKeltnerBands(src, ctx, _AtrPeriod, _Period, 2, _BandsMultiplier, 1);
			IList<double> list3 = KeltnerBands.GenKeltnerBands(src, ctx, _AtrPeriod, _Period, 2, _BandsMultiplier, 2);
			double[] array = new double[list.Count];
			IList<double> list4 = array;
			for (int i = 0; i < count; i++)
			{
				array[i] = 100.0 * (list2[i] - list3[i]) / list[i];
			}
			if (_NormPeriod > 0)
			{
				double[] array2 = new double[count];
				list4 = array2;
				IList<double> list5 = vvSeries.Lowest(array, _NormPeriod);
				IList<double> list6 = vvSeries.Highest(array, _NormPeriod);
				for (int j = 0; j < count; j++)
				{
					double num = list5[j];
					double num2 = list6[j];
					if (num != num2)
					{
						array2[j] = (array[j] - num) / (num2 - num);
					}
					else
					{
						array2[j] = 0.0;
					}
				}
			}
			if (_Smooth > 0)
			{
				list4 = JMA.GenJMA(list4, _Smooth, 0);
			}
			return list4;
		}

		// Token: 0x170002A4 RID: 676
		[HandlerParameter(true, "10", Min = "10", Max = "30", Step = "1")]
		public int AtrPeriod
		{
			// Token: 0x0600084E RID: 2126 RVA: 0x0002344B File Offset: 0x0002164B
			get;
			// Token: 0x0600084F RID: 2127 RVA: 0x00023453 File Offset: 0x00021653
			set;
		}

		// Token: 0x170002A5 RID: 677
		[HandlerParameter(true, "2", Min = "1", Max = "3", Step = "1")]
		public double BandsMultiplier
		{
			// Token: 0x06000850 RID: 2128 RVA: 0x0002345C File Offset: 0x0002165C
			get;
			// Token: 0x06000851 RID: 2129 RVA: 0x00023464 File Offset: 0x00021664
			set;
		}

		// Token: 0x170002A3 RID: 675
		[HandlerParameter(true, "20", Min = "10", Max = "40", Step = "1")]
		public int BandsPeriod
		{
			// Token: 0x0600084C RID: 2124 RVA: 0x0002343A File Offset: 0x0002163A
			get;
			// Token: 0x0600084D RID: 2125 RVA: 0x00023442 File Offset: 0x00021642
			set;
		}

		// Token: 0x170002A8 RID: 680
		public IContext Context
		{
			// Token: 0x06000858 RID: 2136 RVA: 0x000235E6 File Offset: 0x000217E6
			get;
			// Token: 0x06000859 RID: 2137 RVA: 0x000235EE File Offset: 0x000217EE
			set;
		}

		// Token: 0x170002A6 RID: 678
		[HandlerParameter(true, "0", Min = "50", Max = "100", Step = "10")]
		public int NormPeriod
		{
			// Token: 0x06000852 RID: 2130 RVA: 0x0002346D File Offset: 0x0002166D
			get;
			// Token: 0x06000853 RID: 2131 RVA: 0x00023475 File Offset: 0x00021675
			set;
		}

		// Token: 0x170002A7 RID: 679
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000854 RID: 2132 RVA: 0x0002347E File Offset: 0x0002167E
			get;
			// Token: 0x06000855 RID: 2133 RVA: 0x00023486 File Offset: 0x00021686
			set;
		}
	}
}
