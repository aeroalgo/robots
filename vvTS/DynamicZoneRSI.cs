using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200012D RID: 301
	[HandlerCategory("vvRSI"), HandlerName("Dynamic Zone T3 RSI")]
	public class DynamicZoneRSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060008D5 RID: 2261 RVA: 0x000255D0 File Offset: 0x000237D0
		public IList<double> Execute(IList<double> src)
		{
			return DynamicZoneRSI.GenDZRSI(src, this.Context, this.rsiPeriod, this.T3period, this.T3Hot, this.BandsPeriod, this.BandsShift, this.EmaPeriod, this.Chart, this.Output);
		}

		// Token: 0x060008D2 RID: 2258 RVA: 0x00025358 File Offset: 0x00023558
		public static IList<double> GenDZRSI(IList<double> src, IContext ctx, int _rsiPeriod, int _T3period, double _T3Hot, int _BandsPeriod, double _BandsShift, int _EmaPeriod, int _Chart, int _Output)
		{
			int count = src.Count;
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			IList<double> list3 = new double[count];
			IList<double> list4 = T3RSI.GenRSI(src, ctx, _rsiPeriod, _T3period, _T3Hot, false);
			double[] array = new double[_BandsPeriod];
			int num = Math.Max(_rsiPeriod, _BandsPeriod);
			for (int i = num; i < count; i++)
			{
				double num2 = 0.0;
				for (int j = i; j > i - _BandsPeriod; j--)
				{
					array[i - j] = list4[j];
					num2 += list4[j] / (double)_BandsPeriod;
				}
				list[i] = num2 + _BandsShift * DynamicZoneRSI.StDev(array, _BandsPeriod);
				list2[i] = num2 - _BandsShift * DynamicZoneRSI.StDev(array, _BandsPeriod);
				list3[i] = num2;
			}
			IList<double> list5 = EMA.GenEMA(list4, _EmaPeriod);
			if (_Chart > 0)
			{
				IPane pane = ctx.CreatePane("", 35.0, false, false);
				pane.AddList(string.Concat(new string[]
				{
					"DZRsi(",
					_rsiPeriod.ToString(),
					",",
					_BandsPeriod.ToString(),
					",",
					_EmaPeriod.ToString(),
					")"
				}), list4, 0, 16542725, 0, 0);
				pane.AddList("UpZone", list, 0, 1783264, 0, 0);
				pane.AddList("DnZone", list2, 0, 1783264, 0, 0);
				pane.AddList("ma", list3, 0, 1783264, 2, 0);
				pane.AddList("ema", list5, 0, 13369892, 2, 0);
			}
			if (_Output == 1)
			{
				return list;
			}
			if (_Output == 2)
			{
				return list2;
			}
			if (_Output == 3)
			{
				return list3;
			}
			if (_Output == 4)
			{
				return list5;
			}
			return list4;
		}

		// Token: 0x060008D3 RID: 2259 RVA: 0x00025561 File Offset: 0x00023761
		private static double StDev(IList<double> Data, int Per)
		{
			return Math.Sqrt(DynamicZoneRSI.Variance(Data, Per));
		}

		// Token: 0x060008D4 RID: 2260 RVA: 0x00025570 File Offset: 0x00023770
		private static double Variance(IList<double> Data, int Per)
		{
			double num = 0.0;
			double num2 = 0.0;
			for (int i = 0; i < Per; i++)
			{
				num += Data[i];
				num2 += Math.Pow(Data[i], 2.0);
			}
			return (num2 * (double)Per - num * num) / (double)(Per * (Per - 1));
		}

		// Token: 0x170002CE RID: 718
		[HandlerParameter(true, "18", Min = "8", Max = "25", Step = "1")]
		public int BandsPeriod
		{
			// Token: 0x060008C4 RID: 2244 RVA: 0x000252DE File Offset: 0x000234DE
			get;
			// Token: 0x060008C5 RID: 2245 RVA: 0x000252E6 File Offset: 0x000234E6
			set;
		}

		// Token: 0x170002CF RID: 719
		[HandlerParameter(true, "1.3185", Min = "1", Max = "3", Step = "0.1")]
		public double BandsShift
		{
			// Token: 0x060008C6 RID: 2246 RVA: 0x000252EF File Offset: 0x000234EF
			get;
			// Token: 0x060008C7 RID: 2247 RVA: 0x000252F7 File Offset: 0x000234F7
			set;
		}

		// Token: 0x170002D3 RID: 723
		[HandlerParameter(true, "0", NotOptimized = true)]
		public int Chart
		{
			// Token: 0x060008CE RID: 2254 RVA: 0x00025333 File Offset: 0x00023533
			get;
			// Token: 0x060008CF RID: 2255 RVA: 0x0002533B File Offset: 0x0002353B
			set;
		}

		// Token: 0x170002D5 RID: 725
		public IContext Context
		{
			// Token: 0x060008D6 RID: 2262 RVA: 0x00025619 File Offset: 0x00023819
			get;
			// Token: 0x060008D7 RID: 2263 RVA: 0x00025621 File Offset: 0x00023821
			set;
		}

		// Token: 0x170002D0 RID: 720
		[HandlerParameter(true, "5", Min = "1", Max = "25", Step = "1")]
		public int EmaPeriod
		{
			// Token: 0x060008C8 RID: 2248 RVA: 0x00025300 File Offset: 0x00023500
			get;
			// Token: 0x060008C9 RID: 2249 RVA: 0x00025308 File Offset: 0x00023508
			set;
		}

		// Token: 0x170002D4 RID: 724
		[HandlerParameter(true, "0", NotOptimized = true)]
		public int Output
		{
			// Token: 0x060008D0 RID: 2256 RVA: 0x00025344 File Offset: 0x00023544
			get;
			// Token: 0x060008D1 RID: 2257 RVA: 0x0002534C File Offset: 0x0002354C
			set;
		}

		// Token: 0x170002CD RID: 717
		[HandlerParameter(true, "13", Min = "5", Max = "30", Step = "1")]
		public int rsiPeriod
		{
			// Token: 0x060008C2 RID: 2242 RVA: 0x000252CD File Offset: 0x000234CD
			get;
			// Token: 0x060008C3 RID: 2243 RVA: 0x000252D5 File Offset: 0x000234D5
			set;
		}

		// Token: 0x170002D2 RID: 722
		[HandlerParameter(true, "0.618", Min = "0.1", Max = "1", Step = "0.05")]
		public double T3Hot
		{
			// Token: 0x060008CC RID: 2252 RVA: 0x00025322 File Offset: 0x00023522
			get;
			// Token: 0x060008CD RID: 2253 RVA: 0x0002532A File Offset: 0x0002352A
			set;
		}

		// Token: 0x170002D1 RID: 721
		[HandlerParameter(true, "3", Min = "1", Max = "8", Step = "1")]
		public int T3period
		{
			// Token: 0x060008CA RID: 2250 RVA: 0x00025311 File Offset: 0x00023511
			get;
			// Token: 0x060008CB RID: 2251 RVA: 0x00025319 File Offset: 0x00023519
			set;
		}
	}
}
