using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200015F RID: 351
	[HandlerCategory("vvAverages"), HandlerName("EMA_TA")]
	public class EMA_TA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B1E RID: 2846 RVA: 0x0002DAD0 File Offset: 0x0002BCD0
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("ema_ta", new string[]
			{
				this.Alpha.ToString(),
				this.Beta.ToString(),
				src.GetHashCode().ToString()
			}, () => EMA_TA.GenEMA_TA(src, this.Alpha, this.Beta));
		}

		// Token: 0x06000B1D RID: 2845 RVA: 0x0002DA14 File Offset: 0x0002BC14
		public static IList<double> GenEMA_TA(IList<double> src, double _alpha = 0.2, double _beta = 0.3)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			array[0] = src[0];
			array2[0] = src[0];
			for (int i = 1; i < src.Count; i++)
			{
				array2[i] = _alpha * src[i - 1] + (1.0 - _alpha) * (array2[i - 1] + array[i - 1]);
				array[i] = _beta * (array2[i] - array2[i - 1]) + (1.0 - _beta) * array[i - 1];
			}
			return array2;
		}

		// Token: 0x170003A9 RID: 937
		[HandlerParameter(true, "0.2", Min = "0.1", Max = "0.8", Step = "0.1")]
		public double Alpha
		{
			// Token: 0x06000B19 RID: 2841 RVA: 0x0002D9F0 File Offset: 0x0002BBF0
			get;
			// Token: 0x06000B1A RID: 2842 RVA: 0x0002D9F8 File Offset: 0x0002BBF8
			set;
		}

		// Token: 0x170003AA RID: 938
		[HandlerParameter(true, "0.3", Min = "0.1", Max = "0.8", Step = "0.1")]
		public double Beta
		{
			// Token: 0x06000B1B RID: 2843 RVA: 0x0002DA01 File Offset: 0x0002BC01
			get;
			// Token: 0x06000B1C RID: 2844 RVA: 0x0002DA09 File Offset: 0x0002BC09
			set;
		}

		// Token: 0x170003AB RID: 939
		public IContext Context
		{
			// Token: 0x06000B1F RID: 2847 RVA: 0x0002DB4E File Offset: 0x0002BD4E
			get;
			// Token: 0x06000B20 RID: 2848 RVA: 0x0002DB56 File Offset: 0x0002BD56
			set;
		}
	}
}
