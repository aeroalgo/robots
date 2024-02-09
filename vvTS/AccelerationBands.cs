using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000114 RID: 276
	[HandlerCategory("vvBands&Channels"), HandlerName("Acceleration Bands")]
	public class AccelerationBands : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060007C0 RID: 1984 RVA: 0x00021CCA File Offset: 0x0001FECA
		public IList<double> Execute(ISecurity src)
		{
			return AccelerationBands.GenAccelerationBands(src, this.Context, this.MaPeriod, this.MaMode, this.Factor, this.BandMode);
		}

		// Token: 0x060007BF RID: 1983 RVA: 0x00021B84 File Offset: 0x0001FD84
		public static IList<double> GenAccelerationBands(ISecurity sec, IContext ctx, int _MaPeriod, int _MaMode, double _Factor, int _BandMode)
		{
			int count = sec.get_Bars().Count;
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> list = new double[count];
			IList<double> list2 = new double[count];
			for (int i = 0; i < count; i++)
			{
				if (_BandMode == 1)
				{
					list2[i] = highPrices[i] * (1.0 + _Factor * (highPrices[i] - lowPrices[i]) / ((highPrices[i] + lowPrices[i]) / 2.0));
				}
				if (_BandMode == 2)
				{
					list2[i] = lowPrices[i] * (1.0 - _Factor * (highPrices[i] - lowPrices[i]) / ((highPrices[i] + lowPrices[i]) / 2.0));
				}
				if (i < _MaPeriod)
				{
					if (_BandMode == 1)
					{
						list[i] = highPrices[i];
					}
					if (_BandMode == 2)
					{
						list[i] = lowPrices[i];
					}
				}
				else
				{
					list[i] = vvSeries.iMA(list2, list, _MaMode, _MaPeriod, i, 1.0, 1.0);
				}
			}
			return list;
		}

		// Token: 0x17000271 RID: 625
		[HandlerParameter(false, "1", NotOptimized = true, Name = "Band:\n1-top,2-bottom")]
		public int BandMode
		{
			// Token: 0x060007BD RID: 1981 RVA: 0x00021B73 File Offset: 0x0001FD73
			get;
			// Token: 0x060007BE RID: 1982 RVA: 0x00021B7B File Offset: 0x0001FD7B
			set;
		}

		// Token: 0x17000272 RID: 626
		public IContext Context
		{
			// Token: 0x060007C1 RID: 1985 RVA: 0x00021CF0 File Offset: 0x0001FEF0
			get;
			// Token: 0x060007C2 RID: 1986 RVA: 0x00021CF8 File Offset: 0x0001FEF8
			set;
		}

		// Token: 0x17000270 RID: 624
		[HandlerParameter(true, "2", Min = "1", Max = "3", Step = "0.1")]
		public double Factor
		{
			// Token: 0x060007BB RID: 1979 RVA: 0x00021B62 File Offset: 0x0001FD62
			get;
			// Token: 0x060007BC RID: 1980 RVA: 0x00021B6A File Offset: 0x0001FD6A
			set;
		}

		// Token: 0x1700026F RID: 623
		[HandlerParameter(true, "2", Min = "0", Max = "8", Step = "1", Name = "MaMode:\n0-sma,1-ema\n2-smma,3-wma\n4-lsma,5-rema\n6-sinewma\n7-zlema,8-hma")]
		public int MaMode
		{
			// Token: 0x060007B9 RID: 1977 RVA: 0x00021B51 File Offset: 0x0001FD51
			get;
			// Token: 0x060007BA RID: 1978 RVA: 0x00021B59 File Offset: 0x0001FD59
			set;
		}

		// Token: 0x1700026E RID: 622
		[HandlerParameter(true, "20", Min = "10", Max = "20", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x060007B7 RID: 1975 RVA: 0x00021B40 File Offset: 0x0001FD40
			get;
			// Token: 0x060007B8 RID: 1976 RVA: 0x00021B48 File Offset: 0x0001FD48
			set;
		}
	}
}
