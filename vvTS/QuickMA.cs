using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000194 RID: 404
	[HandlerCategory("vvAverages"), HandlerName("QuickMA")]
	public class QuickMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CCF RID: 3279 RVA: 0x00037B5C File Offset: 0x00035D5C
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("quickma", new string[]
			{
				this.Length.ToString(),
				src.GetHashCode().ToString()
			}, () => QuickMA.GenQuickMA(src, this.Length));
		}

		// Token: 0x06000CCE RID: 3278 RVA: 0x00037A5C File Offset: 0x00035C5C
		public static IList<double> GenQuickMA(IList<double> src, int length)
		{
			int count = src.Count;
			double[] array = new double[count];
			for (int i = 0; i < length + 2; i++)
			{
				array[i] = src[i];
			}
			for (int j = length + 2; j < count; j++)
			{
				double num = 0.0;
				double num2 = 0.0;
				double num3 = (double)length / 3.0;
				for (int k = 1; k <= length + 1; k++)
				{
					double num4;
					if ((double)k <= num3)
					{
						num4 = (double)k / num3;
					}
					else
					{
						num4 = (double)(length + 1 - k) / ((double)(length + 1) - num3);
					}
					num += src[j - k - 1] * num4;
					num2 += num4;
				}
				if (num2 != 0.0)
				{
					array[j] = num / num2;
				}
				else
				{
					array[j] = 0.0;
				}
			}
			return array;
		}

		// Token: 0x17000430 RID: 1072
		public IContext Context
		{
			// Token: 0x06000CD0 RID: 3280 RVA: 0x00037BC8 File Offset: 0x00035DC8
			get;
			// Token: 0x06000CD1 RID: 3281 RVA: 0x00037BD0 File Offset: 0x00035DD0
			set;
		}

		// Token: 0x1700042F RID: 1071
		[HandlerParameter(true, "14", Min = "0", Max = "20", Step = "1")]
		public int Length
		{
			// Token: 0x06000CCC RID: 3276 RVA: 0x00037A4A File Offset: 0x00035C4A
			get;
			// Token: 0x06000CCD RID: 3277 RVA: 0x00037A52 File Offset: 0x00035C52
			set;
		}
	}
}
