using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000187 RID: 391
	[HandlerCategory("vvAverages"), HandlerName("MaDev")]
	public class MaDev : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C60 RID: 3168 RVA: 0x00035D04 File Offset: 0x00033F04
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("madev", new string[]
			{
				this.MaPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => MaDev.GenMaDev(src, this.MaPeriod, this.Context));
		}

		// Token: 0x06000C5F RID: 3167 RVA: 0x00035C48 File Offset: 0x00033E48
		public static IList<double> GenMaDev(IList<double> src, int maperiod, IContext context)
		{
			IList<double> data = context.GetData("ma", new string[]
			{
				maperiod.ToString()
			}, () => Series.SMA(src, maperiod));
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = src[i] - data[i];
			}
			return array;
		}

		// Token: 0x1700040D RID: 1037
		public IContext Context
		{
			// Token: 0x06000C61 RID: 3169 RVA: 0x00035D70 File Offset: 0x00033F70
			get;
			// Token: 0x06000C62 RID: 3170 RVA: 0x00035D78 File Offset: 0x00033F78
			set;
		}

		// Token: 0x1700040C RID: 1036
		[HandlerParameter(true, "20", Min = "1", Max = "50", Step = "1")]
		public int MaPeriod
		{
			// Token: 0x06000C5D RID: 3165 RVA: 0x00035C1B File Offset: 0x00033E1B
			get;
			// Token: 0x06000C5E RID: 3166 RVA: 0x00035C23 File Offset: 0x00033E23
			set;
		}
	}
}
