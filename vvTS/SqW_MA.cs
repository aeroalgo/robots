using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200019B RID: 411
	[HandlerCategory("vvAverages"), HandlerName("SqW_MA")]
	public class SqW_MA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D07 RID: 3335 RVA: 0x00039380 File Offset: 0x00037580
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("sqwma", new string[]
			{
				this.Length.ToString(),
				src.GetHashCode().ToString()
			}, () => SqW_MA.GenSqW_MA(src, this.Length));
		}

		// Token: 0x06000D06 RID: 3334 RVA: 0x00039290 File Offset: 0x00037490
		public static IList<double> GenSqW_MA(IList<double> src, int _Length)
		{
			int count = src.Count;
			double[] array = new double[count];
			double num = (double)(_Length * (_Length - 1) / 2);
			double num2 = (double)(_Length * (_Length - 1) * (2 * _Length - 1) / 6);
			for (int i = 0; i < count; i++)
			{
				if (i >= _Length)
				{
					double num3 = 0.0;
					double num4 = 0.0;
					for (int j = 0; j < _Length; j++)
					{
						double num5 = src[i - j];
						num3 += num5;
						num4 += num5 * (double)j;
					}
					double num6 = num2 * (double)_Length - num * num;
					double num7 = (num4 * (double)_Length - num * num3) / num6;
					double num8 = (num3 - num * num7) / (double)_Length;
					array[i] = num8;
				}
				else
				{
					array[i] = src[i];
				}
			}
			return array;
		}

		// Token: 0x1700043F RID: 1087
		public IContext Context
		{
			// Token: 0x06000D08 RID: 3336 RVA: 0x000393EC File Offset: 0x000375EC
			get;
			// Token: 0x06000D09 RID: 3337 RVA: 0x000393F4 File Offset: 0x000375F4
			set;
		}

		// Token: 0x1700043E RID: 1086
		[HandlerParameter(true, "20", Min = "7", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x06000D04 RID: 3332 RVA: 0x0003927D File Offset: 0x0003747D
			get;
			// Token: 0x06000D05 RID: 3333 RVA: 0x00039285 File Offset: 0x00037485
			set;
		}
	}
}
