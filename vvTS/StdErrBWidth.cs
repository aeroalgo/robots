using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000126 RID: 294
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(2), HandlerName("StdErrBandsWidth")]
	public class StdErrBWidth : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000883 RID: 2179 RVA: 0x00023D54 File Offset: 0x00021F54
		public IList<double> Execute(IList<double> src)
		{
			return StdErrBWidth.GenStdErrBWidth(src, this.Context, this.BandsPeriod, this.BandsMultiplier, this.NormPeriod, this.Smooth, this.SmoothPhase);
		}

		// Token: 0x06000882 RID: 2178 RVA: 0x00023AE0 File Offset: 0x00021CE0
		public static IList<double> GenStdErrBWidth(IList<double> src, IContext ctx, int _Period, double _BandsMultiplier, int _NormPeriod, int _Smooth, int _SmoothPhase)
		{
			int count = src.Count;
			double[] array = new double[src.Count];
			IList<double> list = array;
			IList<double> data = ctx.GetData("lrma", new string[]
			{
				_Period.ToString(),
				src.GetHashCode().ToString()
			}, () => LRMA.GenLRMA(src, _Period));
			IList<double> data2 = ctx.GetData("StdErrBand", new string[]
			{
				_Period.ToString(),
				_BandsMultiplier.ToString(),
				1.ToString(),
				src.GetHashCode().ToString()
			}, () => StdErrorBands.GenStdErrBands(src, _Period, _BandsMultiplier, 1, ctx));
			IList<double> data3 = ctx.GetData("StdErrBand", new string[]
			{
				_Period.ToString(),
				_BandsMultiplier.ToString(),
				2.ToString(),
				src.GetHashCode().ToString()
			}, () => StdErrorBands.GenStdErrBands(src, _Period, _BandsMultiplier, 2, ctx));
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = (data2[i] - data3[i]) / data[i] * 100.0;
			}
			if (_NormPeriod > 0)
			{
				double[] array2 = new double[count];
				list = array2;
				IList<double> list2 = Series.Lowest(array, _NormPeriod);
				IList<double> list3 = Series.Highest(array, _NormPeriod);
				for (int j = 0; j < count; j++)
				{
					double num = list2[j];
					double num2 = list3[j];
					if (num != num2)
					{
						array2[j] = 100.0 * ((array[j] - num) / (num2 - num));
					}
					else
					{
						array2[j] = 50.0;
					}
				}
			}
			if (_Smooth > 0)
			{
				list = JMA.GenJMA(list, _Smooth, _SmoothPhase);
			}
			return list;
		}

		// Token: 0x170002B4 RID: 692
		[HandlerParameter(true, "1", Min = "0.5", Max = "3", Step = "0.5")]
		public double BandsMultiplier
		{
			// Token: 0x0600087A RID: 2170 RVA: 0x00023A40 File Offset: 0x00021C40
			get;
			// Token: 0x0600087B RID: 2171 RVA: 0x00023A48 File Offset: 0x00021C48
			set;
		}

		// Token: 0x170002B3 RID: 691
		[HandlerParameter(true, "20", Min = "10", Max = "40", Step = "1")]
		public int BandsPeriod
		{
			// Token: 0x06000878 RID: 2168 RVA: 0x00023A2F File Offset: 0x00021C2F
			get;
			// Token: 0x06000879 RID: 2169 RVA: 0x00023A37 File Offset: 0x00021C37
			set;
		}

		// Token: 0x170002B8 RID: 696
		public IContext Context
		{
			// Token: 0x06000884 RID: 2180 RVA: 0x00023D80 File Offset: 0x00021F80
			get;
			// Token: 0x06000885 RID: 2181 RVA: 0x00023D88 File Offset: 0x00021F88
			set;
		}

		// Token: 0x170002B5 RID: 693
		[HandlerParameter(true, "0", Min = "50", Max = "100", Step = "10")]
		public int NormPeriod
		{
			// Token: 0x0600087C RID: 2172 RVA: 0x00023A51 File Offset: 0x00021C51
			get;
			// Token: 0x0600087D RID: 2173 RVA: 0x00023A59 File Offset: 0x00021C59
			set;
		}

		// Token: 0x170002B6 RID: 694
		[HandlerParameter(true, "0", Min = "0", Max = "30", Step = "1")]
		public int Smooth
		{
			// Token: 0x0600087E RID: 2174 RVA: 0x00023A62 File Offset: 0x00021C62
			get;
			// Token: 0x0600087F RID: 2175 RVA: 0x00023A6A File Offset: 0x00021C6A
			set;
		}

		// Token: 0x170002B7 RID: 695
		[HandlerParameter(true, "0", Min = "-100", Max = "100", Step = "25")]
		public int SmoothPhase
		{
			// Token: 0x06000880 RID: 2176 RVA: 0x00023A73 File Offset: 0x00021C73
			get;
			// Token: 0x06000881 RID: 2177 RVA: 0x00023A7B File Offset: 0x00021C7B
			set;
		}
	}
}
