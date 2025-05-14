using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000189 RID: 393
	[HandlerCategory("vvAverages"), HandlerName("McGinley Dynamic")]
	public class McGinleyDynamic : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C6B RID: 3179 RVA: 0x00035E80 File Offset: 0x00034080
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("McGinleyDynamic", new string[]
			{
				this.Length.ToString(),
				this.Power.ToString(),
				src.GetHashCode().ToString()
			}, () => McGinleyDynamic.GenMcGinleyDynamic(src, this.Context, this.Length, this.Power));
		}

		// Token: 0x06000C6A RID: 3178 RVA: 0x00035DC0 File Offset: 0x00033FC0
		public static IList<double> GenMcGinleyDynamic(IList<double> src, IContext ctx, int _Length = 15, int _Power = 4)
		{
			int count = src.Count;
			double[] array = new double[count];
			_Length = Math.Max(_Length, 1);
			for (int i = 0; i < count; i++)
			{
				if (i < 2)
				{
					array[i] = src[i];
				}
				else
				{
					double num = (double)_Length * Math.Pow(src[i] / array[i - 1], (double)_Power);
					if (num != 0.0)
					{
						array[i] = array[i - 1] + (src[i] - array[i - 1]) / num;
					}
					else
					{
						array[i] = array[i - 1];
					}
				}
			}
			return array;
		}

		// Token: 0x17000410 RID: 1040
		public IContext Context
		{
			// Token: 0x06000C6C RID: 3180 RVA: 0x00035EFE File Offset: 0x000340FE
			get;
			// Token: 0x06000C6D RID: 3181 RVA: 0x00035F06 File Offset: 0x00034106
			set;
		}

		// Token: 0x1700040E RID: 1038
		[HandlerParameter(true, "15", Min = "6", Max = "30", Step = "1")]
		public int Length
		{
			// Token: 0x06000C66 RID: 3174 RVA: 0x00035D9B File Offset: 0x00033F9B
			get;
			// Token: 0x06000C67 RID: 3175 RVA: 0x00035DA3 File Offset: 0x00033FA3
			set;
		}

		// Token: 0x1700040F RID: 1039
		[HandlerParameter(true, "4", Min = "1", Max = "5", Step = "1")]
		public int Power
		{
			// Token: 0x06000C68 RID: 3176 RVA: 0x00035DAC File Offset: 0x00033FAC
			get;
			// Token: 0x06000C69 RID: 3177 RVA: 0x00035DB4 File Offset: 0x00033FB4
			set;
		}
	}
}
