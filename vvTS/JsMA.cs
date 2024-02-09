using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200017D RID: 381
	[HandlerCategory("vvAverages"), HandlerName("JsMA")]
	public class JsMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C0A RID: 3082 RVA: 0x000345F4 File Offset: 0x000327F4
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("jsma", new string[]
			{
				this.length.ToString(),
				src.GetHashCode().ToString()
			}, () => JsMA.GenJsMA(src, this.length));
		}

		// Token: 0x06000C09 RID: 3081 RVA: 0x00034480 File Offset: 0x00032680
		public static IList<double> GenJsMA(IList<double> src, int _len)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double num = 0.45 * (double)(_len - 1) / (0.45 * (double)(_len - 1) + 2.0);
			double num2 = num * num;
			double num3 = (1.0 - num) * (1.0 - num);
			array[0] = (array2[0] = (array3[0] = (array4[0] = (array5[0] = src[0]))));
			for (int i = 1; i < count; i++)
			{
				array[i] = src[i] * (1.0 - num) + array[i - 1] * num;
				array2[i] = (src[i] - array[i]) * (1.0 - num) + array2[i - 1] * num;
				array3[i] = array[i] + array2[i];
				array4[i] = (array3[i] - array5[i - 1]) * num3 + array4[i - 1] * num2;
				array5[i] = array5[i - 1] + array4[i];
			}
			return array5;
		}

		// Token: 0x170003F3 RID: 1011
		public IContext Context
		{
			// Token: 0x06000C0B RID: 3083 RVA: 0x00034660 File Offset: 0x00032860
			get;
			// Token: 0x06000C0C RID: 3084 RVA: 0x00034668 File Offset: 0x00032868
			set;
		}

		// Token: 0x170003F2 RID: 1010
		[HandlerParameter(true, "15", Min = "5", Max = "50", Step = "1")]
		public int length
		{
			// Token: 0x06000C07 RID: 3079 RVA: 0x0003446F File Offset: 0x0003266F
			get;
			// Token: 0x06000C08 RID: 3080 RVA: 0x00034477 File Offset: 0x00032677
			set;
		}
	}
}
