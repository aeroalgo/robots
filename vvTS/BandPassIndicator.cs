using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000012 RID: 18
	[HandlerCategory("vvIndicators"), HandlerName("Bandpass filter")]
	public class BandPassIndicator : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000098 RID: 152 RVA: 0x00003C14 File Offset: 0x00001E14
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("BandpassFilter", new string[]
			{
				this.BandPassPeriod.ToString(),
				this.Delta.ToString(),
				src.GetHashCode().ToString()
			}, () => BandPassIndicator.GenBandpass(src, this.BandPassPeriod, this.Delta));
		}

		// Token: 0x06000097 RID: 151 RVA: 0x00003B10 File Offset: 0x00001D10
		public static IList<double> GenBandpass(IList<double> src, int _BandPassPeriod, double _Delta)
		{
			int count = src.Count;
			double[] array = new double[count];
			double num = 3.1415926535897931;
			double num2 = Math.Cos(2.0 * num / (double)_BandPassPeriod);
			double num3 = 1.0 / Math.Cos(4.0 * num * _Delta / (double)_BandPassPeriod);
			double num4 = num3 - Math.Sqrt(num3 * num3 - 1.0);
			for (int i = 2; i < count; i++)
			{
				array[i] = 0.5 * (1.0 - num4) * (src[i] - src[i - 2]) + num2 * (1.0 + num4) * array[i - 1] - num4 * array[i - 2];
			}
			return array;
		}

		// Token: 0x1700002F RID: 47
		[HandlerParameter(true, "20", Min = "10", Max = "50", Step = "1")]
		public int BandPassPeriod
		{
			// Token: 0x06000093 RID: 147 RVA: 0x00003AEE File Offset: 0x00001CEE
			get;
			// Token: 0x06000094 RID: 148 RVA: 0x00003AF6 File Offset: 0x00001CF6
			set;
		}

		// Token: 0x17000031 RID: 49
		public IContext Context
		{
			// Token: 0x06000099 RID: 153 RVA: 0x00003C92 File Offset: 0x00001E92
			get;
			// Token: 0x0600009A RID: 154 RVA: 0x00003C9A File Offset: 0x00001E9A
			set;
		}

		// Token: 0x17000030 RID: 48
		[HandlerParameter(true, "0.1", Min = "0.1", Max = "1", Step = "0.1")]
		public double Delta
		{
			// Token: 0x06000095 RID: 149 RVA: 0x00003AFF File Offset: 0x00001CFF
			get;
			// Token: 0x06000096 RID: 150 RVA: 0x00003B07 File Offset: 0x00001D07
			set;
		}
	}
}
