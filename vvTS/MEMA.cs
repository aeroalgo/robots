using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200018A RID: 394
	[HandlerCategory("vvAverages"), HandlerName("MEMA")]
	public class MEMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C72 RID: 3186 RVA: 0x00035FC0 File Offset: 0x000341C0
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("mema", new string[]
			{
				this.Factor.ToString(),
				src.GetHashCode().ToString()
			}, () => MEMA.GenMEMA(src, this.Factor));
		}

		// Token: 0x06000C71 RID: 3185 RVA: 0x00035F28 File Offset: 0x00034128
		public static IList<double> GenMEMA(IList<double> src, double factor)
		{
			double[] array = new double[src.Count];
			array[1] = src[1];
			array[0] = src[0];
			for (int i = 2; i < src.Count; i++)
			{
				array[i] = factor * (src[i] - array[i - 1]) + (1.0 - Math.Sqrt(factor)) * (array[i - 1] - array[i - 2]) + array[i - 1];
			}
			return array;
		}

		// Token: 0x17000412 RID: 1042
		public IContext Context
		{
			// Token: 0x06000C73 RID: 3187 RVA: 0x0003602C File Offset: 0x0003422C
			get;
			// Token: 0x06000C74 RID: 3188 RVA: 0x00036034 File Offset: 0x00034234
			set;
		}

		// Token: 0x17000411 RID: 1041
		[HandlerParameter(true, "0.1", Min = "0.01", Max = "1", Step = "0.01")]
		public double Factor
		{
			// Token: 0x06000C6F RID: 3183 RVA: 0x00035F17 File Offset: 0x00034117
			get;
			// Token: 0x06000C70 RID: 3184 RVA: 0x00035F1F File Offset: 0x0003411F
			set;
		}
	}
}
