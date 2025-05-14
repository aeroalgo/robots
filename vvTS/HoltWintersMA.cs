using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000177 RID: 375
	[HandlerCategory("vvAverages"), HandlerName("Holt-Winters MA")]
	public class HoltWintersMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000BD7 RID: 3031 RVA: 0x00032DA8 File Offset: 0x00030FA8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("HoltWintersMA", new string[]
			{
				this.a.ToString(),
				this.b.ToString(),
				this.c.ToString(),
				src.GetHashCode().ToString()
			}, () => HoltWintersMA.GenHoltWintersMA(src, this.a, this.b, this.c));
		}

		// Token: 0x06000BD6 RID: 3030 RVA: 0x00032C40 File Offset: 0x00030E40
		public static IList<double> GenHoltWintersMA(IList<double> src, double _a, double _b, double _c)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			for (int i = 0; i < count; i++)
			{
				if (i < 2)
				{
					array2[i] = 0.0;
					array3[i] = 0.0;
					array4[i] = 0.0;
					array[i] = src[i];
				}
				else
				{
					array2[i] = (1.0 - _a) * (array2[i - 1] + array3[i - 1] + 0.5 * array4[i - 1]) + _a * src[i];
					array3[i] = (1.0 - _b) * (array3[i - 1] + array4[i - 1]) + _b * (array2[i] - array2[i - 1]);
					array4[i] = (1.0 - _c) * array4[i - 1] + _c * (array3[i] - array3[i - 1]);
					array[i] = array2[i] + array3[i] + 0.5 * array4[i];
				}
			}
			return array;
		}

		// Token: 0x170003E2 RID: 994
		[HandlerParameter(true, "0.2", Min = "0.1", Max = "0.3", Step = "0.1")]
		public double a
		{
			// Token: 0x06000BD0 RID: 3024 RVA: 0x00032C0D File Offset: 0x00030E0D
			get;
			// Token: 0x06000BD1 RID: 3025 RVA: 0x00032C15 File Offset: 0x00030E15
			set;
		}

		// Token: 0x170003E3 RID: 995
		[HandlerParameter(true, "0.1", Min = "0.1", Max = "0.3", Step = "0.1")]
		public double b
		{
			// Token: 0x06000BD2 RID: 3026 RVA: 0x00032C1E File Offset: 0x00030E1E
			get;
			// Token: 0x06000BD3 RID: 3027 RVA: 0x00032C26 File Offset: 0x00030E26
			set;
		}

		// Token: 0x170003E4 RID: 996
		[HandlerParameter(true, "0.1", Min = "0.1", Max = "0.3", Step = "0.1")]
		public double c
		{
			// Token: 0x06000BD4 RID: 3028 RVA: 0x00032C2F File Offset: 0x00030E2F
			get;
			// Token: 0x06000BD5 RID: 3029 RVA: 0x00032C37 File Offset: 0x00030E37
			set;
		}

		// Token: 0x170003E5 RID: 997
		public IContext Context
		{
			// Token: 0x06000BD8 RID: 3032 RVA: 0x00032E38 File Offset: 0x00031038
			get;
			// Token: 0x06000BD9 RID: 3033 RVA: 0x00032E40 File Offset: 0x00031040
			set;
		}
	}
}
