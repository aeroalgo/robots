using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000119 RID: 281
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("BBands")]
	public class BBands : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060007E9 RID: 2025 RVA: 0x000220B6 File Offset: 0x000202B6
		public IList<double> Execute(IList<double> src)
		{
			return BBands.GenBBands(src, this.Context, this.BandsPeriod, this.BandsDeviation, this.BandMode, this.MaMethod);
		}

		// Token: 0x060007E6 RID: 2022 RVA: 0x00021F40 File Offset: 0x00020140
		public static IList<double> GenBBands(IList<double> src, IContext ctx, int _BandsPeriod, double _BandsDeviation, int _BandMode, int _MaMethod = 0)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			IList<double> list = AllAverages.Gen_mMA(src, ctx, _MaMethod, _BandsPeriod, _BandsPeriod / 2, 1.0, 1.0);
			for (int i = _BandsPeriod; i < count; i++)
			{
				double num = 0.0;
				int j = i - _BandsPeriod + 1;
				double num2 = list[i];
				while (j <= i)
				{
					double num3 = src[j] - num2;
					num += num3 * num3;
					j++;
				}
				double num4 = _BandsDeviation * Math.Sqrt(num / (double)_BandsPeriod);
				array[i] = num2 + num4;
				array2[i] = num2 - num4;
			}
			if (_BandMode == 1)
			{
				return array;
			}
			if (_BandMode == 2)
			{
				return array2;
			}
			return list;
		}

		// Token: 0x060007E7 RID: 2023 RVA: 0x00022008 File Offset: 0x00020208
		public static IList<double> GenBBands2(IList<double> src, int _BandsPeriod, double _BandsDeviation, int _BandMode)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				double num = BBands.iBBands(src, _BandsPeriod, _BandsDeviation, _BandMode, i);
				array[i] = num;
			}
			return array;
		}

		// Token: 0x060007E8 RID: 2024 RVA: 0x00022044 File Offset: 0x00020244
		public static double iBBands(IList<double> src, int BandsPeriod, double BandsDeviation, int _BandMode, int barNum)
		{
			if (barNum < BandsPeriod)
			{
				BandsPeriod = barNum;
			}
			double num = SMA.iSMA(src, BandsPeriod, barNum);
			double num2 = 0.0;
			int i = barNum - BandsPeriod + 1;
			double num3 = num;
			while (i <= barNum)
			{
				double num4 = src[i] - num3;
				num2 += num4 * num4;
				i++;
			}
			double num5 = BandsDeviation * Math.Sqrt(num2 / (double)BandsPeriod);
			if (_BandMode == 1)
			{
				return num3 + num5;
			}
			if (_BandMode == 2)
			{
				return num3 - num5;
			}
			return num3;
		}

		// Token: 0x1700027D RID: 637
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1", Name = "Mode:\n0-ma,1-top,2-btm")]
		public int BandMode
		{
			// Token: 0x060007E2 RID: 2018 RVA: 0x00021F1D File Offset: 0x0002011D
			get;
			// Token: 0x060007E3 RID: 2019 RVA: 0x00021F25 File Offset: 0x00020125
			set;
		}

		// Token: 0x1700027C RID: 636
		[HandlerParameter(true, "2", Min = "0.5", Max = "3", Step = "0.5")]
		public double BandsDeviation
		{
			// Token: 0x060007E0 RID: 2016 RVA: 0x00021F0C File Offset: 0x0002010C
			get;
			// Token: 0x060007E1 RID: 2017 RVA: 0x00021F14 File Offset: 0x00020114
			set;
		}

		// Token: 0x1700027B RID: 635
		[HandlerParameter(true, "20", Min = "0", Max = "50", Step = "1")]
		public int BandsPeriod
		{
			// Token: 0x060007DE RID: 2014 RVA: 0x00021EFB File Offset: 0x000200FB
			get;
			// Token: 0x060007DF RID: 2015 RVA: 0x00021F03 File Offset: 0x00020103
			set;
		}

		// Token: 0x1700027F RID: 639
		public IContext Context
		{
			// Token: 0x060007EA RID: 2026 RVA: 0x000220DC File Offset: 0x000202DC
			get;
			// Token: 0x060007EB RID: 2027 RVA: 0x000220E4 File Offset: 0x000202E4
			set;
		}

		// Token: 0x1700027E RID: 638
		[HandlerParameter(true, "0", Min = "0", Max = "25", Step = "1", Name = "MaMode:\n0-sma,2-ema,3-wma\n4-jma,5-hma,6-ama\n7-lrma")]
		public int MaMethod
		{
			// Token: 0x060007E4 RID: 2020 RVA: 0x00021F2E File Offset: 0x0002012E
			get;
			// Token: 0x060007E5 RID: 2021 RVA: 0x00021F36 File Offset: 0x00020136
			set;
		}
	}
}
