using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000125 RID: 293
	[HandlerCategory("vvBands&Channels"), HandlerName("StdError Bands")]
	public class StdErrorBands : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000873 RID: 2163 RVA: 0x00023906 File Offset: 0x00021B06
		public IList<double> Execute(IList<double> src1)
		{
			return StdErrorBands.GenStdErrBands(src1, this.Length, this.BandsMultiplier, this.BandMode, this.Context);
		}

		// Token: 0x06000872 RID: 2162 RVA: 0x00023808 File Offset: 0x00021A08
		public static IList<double> GenStdErrBands(IList<double> _price, int _Length, double _BandsMultiplier, int _BandMode, IContext context)
		{
			int count = _price.Count;
			double[] array = new double[count];
			IList<double> list = array;
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			IList<double> price = _price;
			IList<double> data = context.GetData("mid", new string[]
			{
				_Length.ToString(),
				price.GetHashCode().ToString()
			}, () => LRMA.GenLRMA(price, _Length));
			IList<double> list2 = Series.SMA(data, 3);
			for (int i = _Length; i < count; i++)
			{
				double num = StdErrorBands.iStdError(price, _Length, i, ref list);
				array2[i] = list2[i] + _BandsMultiplier * num;
				array3[i] = list2[i] - _BandsMultiplier * num;
			}
			if (_BandMode == 1)
			{
				return array2;
			}
			if (_BandMode == 2)
			{
				return array3;
			}
			return list2;
		}

		// Token: 0x06000874 RID: 2164 RVA: 0x00023928 File Offset: 0x00021B28
		private static double iStdError(IList<double> price, int period, int r, ref IList<double> _workErr)
		{
			_workErr[r] = price[r];
			double num = (double)(period * (period - 1)) * 0.5 / (double)period;
			double num2 = _workErr[r];
			int num3 = 1;
			while (num3 < period && r - num3 >= 0)
			{
				num2 += _workErr[r - num3];
				num3++;
			}
			num2 /= (double)num3;
			double num4 = 0.0;
			double num5 = 0.0;
			double num6 = 0.0;
			for (int i = 0; i < period; i++)
			{
				double num7 = (double)i - num;
				double num8 = _workErr[r - i] - num2;
				num4 += num7 * num7;
				num5 += num8 * num8;
				num6 += num7 * num8;
			}
			double num9 = (num5 - num6 * num6 / num4) / (double)(period - 2);
			if (num9 > 0.0)
			{
				return Math.Sqrt(num9);
			}
			return 0.0;
		}

		// Token: 0x170002B1 RID: 689
		[HandlerParameter(true, "1", Min = "0", Max = "2", Step = "1", Name = "Band:0-centre\n2-top,3-bottom")]
		public int BandMode
		{
			// Token: 0x06000870 RID: 2160 RVA: 0x000237DA File Offset: 0x000219DA
			get;
			// Token: 0x06000871 RID: 2161 RVA: 0x000237E2 File Offset: 0x000219E2
			set;
		}

		// Token: 0x170002B0 RID: 688
		[HandlerParameter(true, "2", Min = "0.5", Max = "3", Step = "0.5")]
		public double BandsMultiplier
		{
			// Token: 0x0600086E RID: 2158 RVA: 0x000237C9 File Offset: 0x000219C9
			get;
			// Token: 0x0600086F RID: 2159 RVA: 0x000237D1 File Offset: 0x000219D1
			set;
		}

		// Token: 0x170002B2 RID: 690
		public IContext Context
		{
			// Token: 0x06000875 RID: 2165 RVA: 0x00023A16 File Offset: 0x00021C16
			get;
			// Token: 0x06000876 RID: 2166 RVA: 0x00023A1E File Offset: 0x00021C1E
			set;
		}

		// Token: 0x170002AF RID: 687
		[HandlerParameter(true, "21", Min = "10", Max = "30", Step = "1")]
		public int Length
		{
			// Token: 0x0600086C RID: 2156 RVA: 0x000237B8 File Offset: 0x000219B8
			get;
			// Token: 0x0600086D RID: 2157 RVA: 0x000237C0 File Offset: 0x000219C0
			set;
		}
	}
}
