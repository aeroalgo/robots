using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200012F RID: 303
	[HandlerCategory("vvRSI"), HandlerName("RSIFilter")]
	public class RSIFilter : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060008EF RID: 2287 RVA: 0x00025B57 File Offset: 0x00023D57
		public IList<double> Execute(IList<double> src)
		{
			return RSIFilter.GenRSIFilter(src, this.RsiPeriod, this.Smooth, this.UpTrendLevel, this.DnTrendLevel, this.Context);
		}

		// Token: 0x060008EE RID: 2286 RVA: 0x00025A00 File Offset: 0x00023C00
		public static IList<double> GenRSIFilter(IList<double> src, int rsiperiod, int smooth, int _UpTrendLevel, int _DnTrendLevel, IContext ctx)
		{
			int arg_2B_0 = src.Count;
			IList<double> list = new double[src.Count];
			IList<double> data = ctx.GetData("rsi", new string[]
			{
				rsiperiod.ToString(),
				smooth.ToString(),
				src.GetHashCode().ToString()
			}, () => RSI.GenRSI(src, rsiperiod, 0, smooth, 100, false));
			int num = 0;
			for (int i = 0; i < src.Count; i++)
			{
				if (data[i] > 70.0)
				{
					num = 1;
				}
				if (data[i] < 30.0)
				{
					num = -1;
				}
				if (num > 0)
				{
					if (data[i] > (double)_UpTrendLevel)
					{
						list[i] = 1.0;
					}
					else
					{
						list[i] = 0.0;
						num = 0;
					}
				}
				if (num < 0)
				{
					if (data[i] < (double)_DnTrendLevel)
					{
						list[i] = -1.0;
					}
					else
					{
						list[i] = 0.0;
						num = 0;
					}
				}
			}
			return list;
		}

		// Token: 0x170002DF RID: 735
		public IContext Context
		{
			// Token: 0x060008F0 RID: 2288 RVA: 0x00025B7D File Offset: 0x00023D7D
			get;
			// Token: 0x060008F1 RID: 2289 RVA: 0x00025B85 File Offset: 0x00023D85
			set;
		}

		// Token: 0x170002DE RID: 734
		[HandlerParameter(true, "60", Min = "50", Max = "70", Step = "1")]
		public int DnTrendLevel
		{
			// Token: 0x060008EC RID: 2284 RVA: 0x000259C9 File Offset: 0x00023BC9
			get;
			// Token: 0x060008ED RID: 2285 RVA: 0x000259D1 File Offset: 0x00023BD1
			set;
		}

		// Token: 0x170002DB RID: 731
		[HandlerParameter(true, "9", Min = "0", Max = "60", Step = "1")]
		public int RsiPeriod
		{
			// Token: 0x060008E6 RID: 2278 RVA: 0x00025996 File Offset: 0x00023B96
			get;
			// Token: 0x060008E7 RID: 2279 RVA: 0x0002599E File Offset: 0x00023B9E
			set;
		}

		// Token: 0x170002DC RID: 732
		[HandlerParameter(true, "3", Min = "0", Max = "30", Step = "1")]
		public int Smooth
		{
			// Token: 0x060008E8 RID: 2280 RVA: 0x000259A7 File Offset: 0x00023BA7
			get;
			// Token: 0x060008E9 RID: 2281 RVA: 0x000259AF File Offset: 0x00023BAF
			set;
		}

		// Token: 0x170002DD RID: 733
		[HandlerParameter(true, "40", Min = "30", Max = "50", Step = "1")]
		public int UpTrendLevel
		{
			// Token: 0x060008EA RID: 2282 RVA: 0x000259B8 File Offset: 0x00023BB8
			get;
			// Token: 0x060008EB RID: 2283 RVA: 0x000259C0 File Offset: 0x00023BC0
			set;
		}
	}
}
