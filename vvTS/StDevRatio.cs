using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200005B RID: 91
	[HandlerCategory("vvIndicators"), HandlerName("StDev Ratio")]
	public class StDevRatio : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000344 RID: 836 RVA: 0x00012CB4 File Offset: 0x00010EB4
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("StDevRatio", new string[]
			{
				this.ShortStDevPeriod.ToString(),
				this.LongStDevPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => StDevRatio.GenSTDEVratio(src, this.ShortStDevPeriod, this.LongStDevPeriod, this.Context));
		}

		// Token: 0x06000343 RID: 835 RVA: 0x00012B80 File Offset: 0x00010D80
		public static IList<double> GenSTDEVratio(IList<double> src, int shortstdevperiod, int longstdevperiod, IContext ctx)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> data = ctx.GetData("stdev", new string[]
			{
				shortstdevperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.StDev(src, shortstdevperiod));
			IList<double> data2 = ctx.GetData("stdev", new string[]
			{
				longstdevperiod.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.StDev(src, longstdevperiod));
			for (int i = 0; i < count; i++)
			{
				array[i] = data[i] / data2[i];
			}
			return array;
		}

		// Token: 0x17000119 RID: 281
		public IContext Context
		{
			// Token: 0x06000345 RID: 837 RVA: 0x00012D32 File Offset: 0x00010f32
			get;
			// Token: 0x06000346 RID: 838 RVA: 0x00012D3A File Offset: 0x00010F3A
			set;
		}

		// Token: 0x17000118 RID: 280
		[HandlerParameter(true, "50", Min = "10", Max = "100", Step = "1")]
		public int LongStDevPeriod
		{
			// Token: 0x06000341 RID: 833 RVA: 0x00012B40 File Offset: 0x00010D40
			get;
			// Token: 0x06000342 RID: 834 RVA: 0x00012B48 File Offset: 0x00010D48
			set;
		}

		// Token: 0x17000117 RID: 279
		[HandlerParameter(true, "5", Min = "1", Max = "30", Step = "1")]
		public int ShortStDevPeriod
		{
			// Token: 0x0600033F RID: 831 RVA: 0x00012B2F File Offset: 0x00010D2F
			get;
			// Token: 0x06000340 RID: 832 RVA: 0x00012B37 File Offset: 0x00010D37
			set;
		}
	}
}
